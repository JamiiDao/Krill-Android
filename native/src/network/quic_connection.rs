use std::{
    net::{IpAddr, SocketAddr},
    str::FromStr,
};

use bitcode::{Decode, DecodeOwned, Encode};
use krill_common::{QuicCommon, QuicTransmissionEncoder, QuicTransmissionError, RandomBytes};

use quinn::{
    crypto::rustls::{NoInitialCipherSuite, QuicClientConfig},
    ClientConfig, Endpoint,
};
use rustls::{
    client::danger,
    crypto::{verify_tls12_signature, verify_tls13_signature, CryptoProvider},
    pki_types::{CertificateDer, ServerName, UnixTime},
    DigitallySignedStruct, SignatureScheme,
};
use std::sync::Arc;

pub struct QuicClient;

impl QuicClient {
    pub async fn connect<T: Encode + DecodeOwned>(
        sld_tld: &str,
        payload: &(impl Encode + Decode<'static>),
    ) -> Result<T, QuicTransmissionError> {
        let (mut send, mut recv) = Self::setup_connect(sld_tld).await?.open_bi().await?;

        let encoded_payload = bitcode::encode(payload);

        let len = (encoded_payload.len() as u32).to_be_bytes();

        send.write_all(&len).await?;

        send.write_all(&encoded_payload).await?;

        // signal end of request
        send.finish()?;

        let mut len_buf = [0u8; 4];

        recv.read_exact(&mut len_buf).await?;

        let len = u32::from_be_bytes(len_buf) as usize;

        let mut response = vec![0u8; len];

        recv.read_exact(&mut response).await?;

        if response.is_empty() {
            return Err(QuicTransmissionError::InvalidResponsePayload);
        }

        if response[0] == 0 {
            QuicTransmissionEncoder::decode_success::<T>(&response[1..])
        } else {
            Err(QuicTransmissionEncoder::decode_failure(&response[1..]))
        }
    }

    pub async fn setup_connect(sld_tld: &str) -> Result<quinn::Connection, QuicTransmissionError> {
        let (mut socket_addr, server_name, classification) = Self::resolve(sld_tld).await?;

        let client_config = if classification == IpClassification::Internet {
            quinn::ClientConfig::try_with_platform_verifier()?
        } else {
            Self::configure_client()
                .map_err(|error| QuicTransmissionError::Rustls(error.to_string()))?
        };

        let mut random_u16: u16;
        let mut endpoint: Endpoint;

        loop {
            let random_inner = RandomBytes::<2>::generate().map_err(|error| {
                crate::ClientUtils::log_to_logcat(&format!(
                    "Unable to generate randomness for port! {:?}",
                    error
                ));

                QuicTransmissionError::Randomness
            })?;

            random_u16 = u16::from_le_bytes(*random_inner.take());

            if random_u16 < 1024 {
                continue;
            } else {
                let socket = SocketAddr::new(QuicCommon::UNSPECIFIED_V4, random_u16);
                match Endpoint::client(socket) {
                    Ok(value) => {
                        endpoint = value;
                        break;
                    }
                    Err(error) => {
                        if error.kind() == std::io::ErrorKind::AddrInUse {
                            continue;
                        } else {
                            return Err(error.into());
                        }
                    }
                }
            }
        }

        socket_addr.set_port(QuicCommon::SERVER_PORT);

        // TODO set better timeout

        endpoint.set_default_client_config(client_config);
        // Connect to the server passing in the server name which is supposed to be in the server certificate.

        Ok(endpoint.connect(socket_addr, &server_name)?.await?)
    }

    fn configure_client() -> Result<ClientConfig, NoInitialCipherSuite> {
        let crypto = rustls::ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(SkipServerVerification::new())
            .with_no_client_auth();

        Ok(ClientConfig::new(Arc::new(QuicClientConfig::try_from(
            crypto,
        )?)))
    }

    // TODO use krill-common for this
    pub async fn resolve(sld_tld: &str) -> std::io::Result<(SocketAddr, String, IpClassification)> {
        let inner_sld_tld = sld_tld.to_string() + ":0";

        let (addr, classification) = if let Ok(addr) = SocketAddr::from_str(&inner_sld_tld) {
            (addr, Self::classify_ip(addr.ip()))
        } else {
            let resolved =
                async_net::resolve(&inner_sld_tld)
                    .await?
                    .first()
                    .cloned()
                    .ok_or::<std::io::Error>(std::io::ErrorKind::NetworkUnreachable.into())?;

            (resolved, Self::classify_ip(resolved.ip()))
        };

        let sni_name = if classification == IpClassification::Internet {
            sld_tld
        } else {
            classification.server_name()
        };

        Ok((addr, sni_name.to_string(), classification))
    }

    // TODO check for and IP crate for better checks especially for broadcasts.
    // Try `ipnet` or `smoltcp` crates
    pub fn classify_ip(ip: IpAddr) -> IpClassification {
        match ip {
            IpAddr::V4(v4) => {
                if v4.is_loopback() {
                    return IpClassification::LocalHost;
                }

                if v4.is_private() {
                    return IpClassification::LAN;
                }

                if v4.is_link_local() {
                    return IpClassification::LinkLocal;
                }

                if v4.is_multicast() {
                    return IpClassification::Multicast;
                }

                IpClassification::Internet
            }

            IpAddr::V6(v6) => {
                if v6.is_loopback() {
                    return IpClassification::LocalHost;
                }

                if v6.is_unicast_link_local() {
                    return IpClassification::LinkLocal;
                }

                if v6.is_multicast() {
                    return IpClassification::Multicast;
                }

                if v6.is_unique_local() {
                    return IpClassification::LAN;
                }

                IpClassification::Internet
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, uniffi::Enum)]
pub enum IpClassification {
    LocalHost,
    LAN,
    LinkLocal,
    Multicast,
    Internet,
}

impl IpClassification {
    #[uniffi::method]
    pub fn context_string(&self) -> &'static str {
        match self {
            Self::LocalHost => "localhost",
            Self::LAN => "Local Area Network (LAN)",
            Self::LinkLocal => "Link-Local",
            Self::Multicast => "Multicast",
            Self::Internet => "Internet",
        }
    }

    pub fn server_name(&self) -> &'static str {
        match self {
            Self::Internet => "Internet",
            _ => QuicCommon::SERVER_NAME_QUIC_DEBUG,
        }
    }
}

// Implementation of `ServerCertVerifier` that verifies everything as trustworthy.
#[derive(Debug)]
struct SkipServerVerification(Arc<CryptoProvider>);

impl SkipServerVerification {
    fn new() -> Arc<Self> {
        Arc::new(Self(Arc::new(rustls::crypto::ring::default_provider())))
    }
}

impl danger::ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp: &[u8],
        _now: UnixTime,
    ) -> Result<danger::ServerCertVerified, rustls::Error> {
        Ok(danger::ServerCertVerified::assertion())
    }
    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<danger::HandshakeSignatureValid, rustls::Error> {
        verify_tls12_signature(
            message,
            cert,
            dss,
            &self.0.signature_verification_algorithms,
        )
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<danger::HandshakeSignatureValid, rustls::Error> {
        verify_tls13_signature(
            message,
            cert,
            dss,
            &self.0.signature_verification_algorithms,
        )
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.0.signature_verification_algorithms.supported_schemes()
    }
}
