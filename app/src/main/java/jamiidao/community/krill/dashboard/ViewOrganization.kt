package jamiidao.community.krill.dashboard

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.size
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.navigation.NavController
import coil.ImageLoader
import coil.compose.AsyncImage
import coil.decode.SvgDecoder
import jamiidao.community.krill.DashboardRoute
import jamiidao.community.krill.R
import jamiidao.community.krill.RustFfiException
import jamiidao.community.krill.RustTypeOrganizationInfo
import jamiidao.community.krill.RustTypeStoredOrgInfoMetadata
import jamiidao.community.krill.ViewOrganizationRoute
import jamiidao.community.krill.components.AppText
import jamiidao.community.krill.components.GlassButton
import jamiidao.community.krill.components.KrillGlassSurface
import jamiidao.community.krill.components.KrillLogo
import jamiidao.community.krill.components.KrillStripedLoader
import jamiidao.community.krill.components.ShowErrorAsBottomSheet
import jamiidao.community.krill.rustFnFetchOrgInfo
import jamiidao.community.krill.rustFnLoadStoredOrganizationInfo
import jamiidao.community.krill.ui.theme.CadmiumOrange
import jamiidao.community.krill.ui.theme.bungeeHairlineFamily
import jamiidao.community.krill.ui.theme.commitMonoFamily


@Composable
fun ViewOrganizationView(navController: NavController, sldTld: String) {
    val state = remember { mutableStateOf(ComponentState.Fetching) }
    val orgInfo = remember { mutableStateOf<RustTypeStoredOrgInfoMetadata?>(null) }
    val error = remember { mutableStateOf<String?>(null) }

    LaunchedEffect(Unit) {
        try {
            orgInfo.value = rustFnLoadStoredOrganizationInfo(sldTld)

            state.value = ComponentState.Done
        } catch (e: RustFfiException) {
            error.value = e.uiMessage()
        }
    }

    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center,
        modifier = Modifier.fillMaxSize()
    ) {
        when (state.value) {
            ComponentState.Fetching -> {
                KrillLogo()
                Spacer(Modifier.height(10.dp))
                AppText(
                    textContent = "Loading $sldTld", fontSize = 18.sp,
                    fontWeight = FontWeight.Normal, fontFamily = commitMonoFamily,
                    color = CadmiumOrange
                )
                Spacer(Modifier.height(5.dp))
                KrillStripedLoader()
            }

            ComponentState.Done -> {
                orgInfo.value?.let {
                    OrgDetails(navController, it)
                } ?: Column(
                    horizontalAlignment = Alignment.CenterHorizontally,
                    verticalArrangement = Arrangement.SpaceAround,
                    modifier = Modifier.fillMaxSize()
                ) {
                    AppText(textContent = "Organization Not Found")
                    Spacer(Modifier.height(30.dp))
                }
            }
        }
    }

    error.value?.let {
        ShowErrorAsBottomSheet(
            title = "Get Organization",
            error = it,
            imageID = R.drawable.error,
            imageDescription = "",
            buttonTextContent = "Ok",
            callback = {
                navController.navigate(DashboardRoute) {
                    popUpTo(navController.currentDestination?.id ?: return@navigate) {
                        inclusive = false
                    }
                }
            },
        )
    }
}

@Composable
fun OrgDetails(navController: NavController, metadata: RustTypeStoredOrgInfoMetadata) {
    val context = LocalContext.current

    val imageLoader = ImageLoader.Builder(context)
        .components {
            add(SvgDecoder.Factory())
        }
        .build()
    Column(
        verticalArrangement = Arrangement.SpaceAround,
        horizontalAlignment = Alignment.CenterHorizontally,
        modifier = Modifier.fillMaxSize()
    ) {
        KrillGlassSurface(
            modifier = Modifier.fillMaxWidth(.8f),
            percentage = 10,
            content = {
                Column(
                    horizontalAlignment = Alignment.CenterHorizontally,
                    verticalArrangement = Arrangement.Center,
                    modifier = Modifier
                ) {
                    AsyncImage(
                        model = metadata.logoIcon,
                        imageLoader = imageLoader,
                        contentDescription = null,
                        modifier = Modifier.size(200.dp)
                    )
                    Spacer(Modifier.height(10.dp))
                    AppText(
                        textContent = metadata.orgName,
                        fontSize = 28.sp,
                        fontFamily = bungeeHairlineFamily,
                        fontWeight = FontWeight.Black
                    )
                    Spacer(Modifier.height(5.dp))
                    AppText(
                        textContent = metadata.sldTld,
                        fontSize = 18.sp,
                        fontFamily = commitMonoFamily,
                        color = CadmiumOrange
                    )
                    Spacer(Modifier.height(20.dp))
                    AppText(
                        textContent = metadata.supportMail,
                        fontSize = 18.sp,
                        fontFamily = commitMonoFamily,
                    )
                    Spacer(Modifier.height(20.dp))
                }
            })
    }
}

enum class ComponentState {
    Fetching,
    Done
}

