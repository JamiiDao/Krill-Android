package jamiidao.community.krill

import android.Manifest
import android.content.Intent
import android.content.pm.PackageManager
import android.net.Uri
import android.provider.Settings
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.annotation.OptIn
import androidx.camera.core.CameraSelector
import androidx.camera.core.ExperimentalGetImage
import androidx.camera.core.ImageAnalysis
import androidx.camera.core.Preview
import androidx.camera.lifecycle.ProcessCameraProvider
import androidx.camera.view.PreviewView
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.viewinterop.AndroidView
import androidx.core.app.ActivityCompat.shouldShowRequestPermissionRationale
import androidx.core.content.ContextCompat
import androidx.lifecycle.compose.LocalLifecycleOwner
import androidx.navigation.NavController
import com.google.mlkit.vision.barcode.BarcodeScannerOptions
import com.google.mlkit.vision.barcode.BarcodeScanning
import com.google.mlkit.vision.barcode.common.Barcode
import com.google.mlkit.vision.common.InputImage
import jamiidao.community.krill.components.AppText
import jamiidao.community.krill.components.GlassButton
import jamiidao.community.krill.components.KrillLogo
import jamiidao.community.krill.dashboard.ROUTE_SCAN_QR
import java.util.concurrent.Executors
import androidx.core.net.toUri
import jamiidao.community.krill.components.ShowErrorAsBottomSheet
import jamiidao.community.krill.dashboard.ROUTE_GROUPS


@Composable
fun ScanQR(
    activity: MainActivity,
    navController: NavController,
    bottomNavController: NavController
) {
    val context = LocalContext.current
    val cameraPermState = remember {
        ContextCompat.checkSelfPermission(context, Manifest.permission.CAMERA)
    }

    if (cameraPermState != PackageManager.PERMISSION_GRANTED) {
        GrantCameraPermission(
            activity = activity,
            navController = navController,
            bottomNavController = bottomNavController
        )
    } else {
        Column(
            verticalArrangement = Arrangement.SpaceAround,
            horizontalAlignment = Alignment.CenterHorizontally,
            modifier = Modifier.fillMaxSize()
        ) {
            QRScannerScreen(navController, bottomNavController)
        }
    }
}

@OptIn(ExperimentalGetImage::class)
@Composable
private fun QRScannerScreen(navController: NavController, bottomNavController: NavController) {

    val context = LocalContext.current
    val lifecycleOwner = LocalLifecycleOwner.current

    var scannedValue by remember { mutableStateOf<String?>(null) }
    var hasNavigated by remember { mutableStateOf(false) }
    var errorExists by remember { mutableStateOf<String?>(null) }

    val previewView = remember { PreviewView(context) }

    val cameraProviderFuture = remember { ProcessCameraProvider.getInstance(context) }
    val executor = remember { Executors.newSingleThreadExecutor() }

    DisposableEffect(lifecycleOwner) {

        val listener = Runnable {
            val cameraProvider = cameraProviderFuture.get()

            val preview = Preview.Builder().build().apply {
                surfaceProvider = previewView.surfaceProvider
            }

            val analyzer = ImageAnalysis.Builder()
                .setBackpressureStrategy(ImageAnalysis.STRATEGY_KEEP_ONLY_LATEST)
                .build()

            val scanner = BarcodeScanning.getClient()

            analyzer.setAnalyzer(executor) { imageProxy ->

                val mediaImage = imageProxy.image
                if (mediaImage == null) {
                    imageProxy.close()
                    return@setAnalyzer
                }

                val inputImage = InputImage.fromMediaImage(
                    mediaImage,
                    imageProxy.imageInfo.rotationDegrees
                )

                scanner.process(inputImage)
                    .addOnSuccessListener { barcodes ->

                        if (hasNavigated) {
                            imageProxy.close()
                            return@addOnSuccessListener
                        }

                        val raw = barcodes.firstOrNull()?.rawValue

                        if (!raw.isNullOrBlank()) {
                            if (isValidDeeplink(raw)) {
                                hasNavigated = true
                                scannedValue = raw
                            }
                        }
                    }
                    .addOnFailureListener {
                        errorExists = it.message
                    }
                    .addOnCompleteListener {
                        imageProxy.close()
                    }
            }

            try {
                cameraProvider.unbindAll()
                cameraProvider.bindToLifecycle(
                    lifecycleOwner,
                    CameraSelector.DEFAULT_BACK_CAMERA,
                    preview,
                    analyzer
                )
            } catch (e: Exception) {
                errorExists = e.toString()
            }
        }

        cameraProviderFuture.addListener(
            listener,
            ContextCompat.getMainExecutor(context)
        )

        onDispose {
            try {
                val provider = cameraProviderFuture.get()
                provider.unbindAll()
            } catch (e: Exception) {
                errorExists = e.toString()
            }
            executor.shutdown()
        }
    }

    AndroidView(
        factory = { previewView },
        modifier = Modifier
            .fillMaxWidth()
            .height(350.dp)
    )

    Column(
        modifier = Modifier.fillMaxSize(),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Top
    ) {

        AppText(
            textContent = "Scan Krill QR Code",
        )

        Spacer(modifier = Modifier.height(24.dp))

        if (errorExists != null) {
            app_log("Encountered error while scanning QR Code: $errorExists")

            ShowErrorAsBottomSheet(
                title = "Scan QR",
                error = "Unable to Scan QR code",
                imageID = R.drawable.qrcode,
                imageDescription = "Krill Logo",
                buttonTextContent = "Ok",
                callback = {
                    bottomNavController.navigate(ROUTE_GROUPS)
                }
            )
        }
    }

    LaunchedEffect(scannedValue) {
        val value = scannedValue ?: return@LaunchedEffect

        if (!hasNavigated) return@LaunchedEffect

        val prefixRemoved = value
            .removePrefix("krill://")

        navController.navigate(prefixRemoved)
    }
}


@Composable
fun GrantCameraPermission(
    activity: MainActivity,
    navController: NavController,
    bottomNavController: NavController,
) {

    val context = LocalContext.current
    val permission = Manifest.permission.CAMERA

    val permanentlyDeniedText =
        "You have permanently denied this app from accessing the camera! \nGo to the settings of this app to enable camera permission.\n\n This will allow you to scan QR Codes!"
    val allowPermissionsText =
        "Allow this app to access the camera to scan QR Codes!"

    val innerPermissionState = remember { mutableStateOf(InnerPermissionState.Denied) }

    val permissionLauncher = rememberLauncherForActivityResult(
        contract = ActivityResultContracts.RequestPermission(),
        onResult = { isGranted ->
            if (!isGranted) {
                if (shouldShowRequestPermissionRationale(activity, permission)) {
                    innerPermissionState.value = InnerPermissionState.ShowRationale
                } else {
                    innerPermissionState.value = InnerPermissionState.PermanentlyDenied
                }
            } else {
                bottomNavController.navigate(ROUTE_SCAN_QR)
            }

        }
    )

    when (innerPermissionState.value) {
        InnerPermissionState.Denied -> {
            NotificationView(
                navController,
                contentText = allowPermissionsText,
                buttonText = "Grant Access",
                buttonCallback = {
                    permissionLauncher.launch(permission)
                },
            )
        }

        InnerPermissionState.PermanentlyDenied -> {
            NotificationView(
                navController,
                contentText = permanentlyDeniedText,
                buttonText = "App Settings",
                buttonCallback = {
                    val intent = Intent().apply {
                        action = Settings.ACTION_APPLICATION_DETAILS_SETTINGS
                        data = "package:${context.packageName}".toUri()
                    }
                    context.startActivity(intent)
                }
            )
        }

        InnerPermissionState.ShowRationale -> NotificationView(
            navController,
            contentText = allowPermissionsText,
            buttonText = "Grant Access",
            buttonCallback = {
                permissionLauncher.launch(permission)
            }
        )
    }
}


@Composable
fun NotificationView(
    navController: NavController,
    contentText: String,
    buttonText: String,
    buttonCallback: () -> Unit,
) {
    Column(
        modifier = Modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState()),
        verticalArrangement = Arrangement.SpaceAround,
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        Column(
            verticalArrangement = Arrangement.Center,
            horizontalAlignment = Alignment.CenterHorizontally,
            modifier = Modifier
                .weight(1f)
                .fillMaxWidth(.9f)
        ) {
            KrillLogo()
            Spacer(Modifier.height(50.dp))
            AppText(
                textContent = contentText, fontSize = 25.sp
            )
        }

        Row(
            modifier = Modifier
                .weight(.4f)
                .fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.SpaceBetween
        ) {
            GlassButton(
                callback = {
                    navController.navigate(DashboardRoute) {
                        popUpTo(DashboardRoute) { inclusive = false }
                    }
                },
                textContent = "Skip", width = .4f,
            )

            GlassButton(
                callback =
                    buttonCallback,
                textContent = buttonText,
                width = 1f, filled = true
            )
        }
    }
}

enum class InnerPermissionState { Denied, PermanentlyDenied, ShowRationale }