package jamiidao.community.krill.deeplinks

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.size
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.lifecycleScope
import androidx.navigation.NavController
import coil.ImageLoader
import coil.compose.AsyncImage
import coil.decode.SvgDecoder
import com.google.firebase.messaging.FirebaseMessaging
import jamiidao.community.krill.DashboardRoute
import jamiidao.community.krill.R
import jamiidao.community.krill.RustFfiException
import jamiidao.community.krill.RustTypeOrganizationInfo
import jamiidao.community.krill.ViewGroupActivitiesRoute
import jamiidao.community.krill.appStoragePath
import jamiidao.community.krill.app_log
import jamiidao.community.krill.components.AppText
import jamiidao.community.krill.components.GlassButton
import jamiidao.community.krill.components.KrillGlassSurface
import jamiidao.community.krill.components.KrillLogo
import jamiidao.community.krill.components.KrillStripedLoader
import jamiidao.community.krill.components.ShowErrorAsBottomSheet
import jamiidao.community.krill.rustFnFetchOrgInfo
import jamiidao.community.krill.rustFnJoin
import jamiidao.community.krill.rustFnSetFcmToken
import jamiidao.community.krill.ui.theme.CadmiumOrange
import jamiidao.community.krill.ui.theme.bungeeHairlineFamily
import jamiidao.community.krill.ui.theme.commitMonoFamily
import kotlinx.coroutines.launch
import kotlinx.coroutines.tasks.await


@Composable
fun JoinOrganization(
    navController: NavController,
    sldTld: String
) {

    val orgInfo = remember { mutableStateOf<RustTypeOrganizationInfo?>(null) }

    val context = LocalContext.current

    val error = remember { mutableStateOf<String?>(null) }

    val imageLoader = ImageLoader.Builder(context)
        .components {
            add(SvgDecoder.Factory())
        }
        .build()

    LaunchedEffect(Unit) {
        try {
            orgInfo.value = rustFnFetchOrgInfo(sldTld)

        } catch (e: RustFfiException) {
            error.value = e.uiMessage()
        }
    }

    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center,
        modifier = Modifier.fillMaxSize()
    ) {
        orgInfo.value?.let {
            DisplayOrgInfo(navController, sldTld, it, imageLoader)
        } ?: Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.Center,
            modifier = Modifier.fillMaxSize()
        ) {
            error.value?.let {
                ShowErrorAsBottomSheet(
                    title = sldTld,
                    error = it,
                    imageID = R.drawable.members,
                    buttonTextContent = "Ok",
                    imageDescription = "",
                    callback = {
                        navController.navigate(DashboardRoute);
                    }
                )
            } ?: KrillLogo()
            Spacer(Modifier.height(10.dp))
            AppText(
                textContent = "Accessing $sldTld", fontSize = 18.sp,
                fontWeight = FontWeight.Normal, fontFamily = commitMonoFamily,
                color = CadmiumOrange
            )
            Spacer(Modifier.height(5.dp))
            KrillStripedLoader()
        }

    }
}

@Composable
fun DisplayOrgInfo(
    navController: NavController,
    sldTld: String, info: RustTypeOrganizationInfo, imageLoader: ImageLoader
) {
    val appStoragePathFetched = appStoragePath(LocalContext.current)

    val scope = rememberCoroutineScope()
    val error = remember { mutableStateOf<RustFfiException?>(null) }

    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.SpaceAround,
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
                        model = info.logoIcon,
                        imageLoader = imageLoader,
                        contentDescription = null,
                        modifier = Modifier.size(200.dp)
                    )
                    Spacer(Modifier.height(10.dp))
                    AppText(
                        textContent = info.name,
                        fontSize = 28.sp,
                        fontFamily = bungeeHairlineFamily,
                        fontWeight = FontWeight.Black
                    )
                    Spacer(Modifier.height(5.dp))
                    AppText(
                        textContent = sldTld,
                        fontSize = 18.sp,
                        fontFamily = commitMonoFamily,
                        color = CadmiumOrange
                    )
                    Spacer(Modifier.height(20.dp))
                    AppText(
                        textContent = info.supportMail,
                        fontSize = 18.sp,
                        fontFamily = commitMonoFamily,
                    )
                    Spacer(Modifier.height(20.dp))
                }
            }
        )

        Row(
            horizontalArrangement = Arrangement.SpaceAround,
            verticalAlignment = Alignment.CenterVertically,
            modifier = Modifier.fillMaxWidth()
        ) {
            GlassButton(
                textContent = "Reject",
                width = 0.4f,
                callback = {
                    navController.navigate(DashboardRoute) {
                        popUpTo(0)
                    }
                }
            )

            GlassButton(
                textContent = "Join",
                width = 0.8f, filled = true,
                callback = {
                    app_log("JOIN $appStoragePathFetched")
                    scope.launch {
                        try {
                            val token = FirebaseMessaging.getInstance().token.await()

                            rustFnJoin(
                                appStoragePath = appStoragePathFetched,
                                sldTld,
                                info,
                                token
                            )

                            navController.navigate(ViewGroupActivitiesRoute(sldTld)) {
                                popUpTo(0)
                            }
                        } catch (e: RustFfiException) {
                            error.value = e
                        }
                    }
                }
            )
        }

        error.value?.let {
            ShowErrorAsBottomSheet(
                title = sldTld,
                error = it.uiMessage(),
                imageID = R.drawable.error,
                imageDescription = "",
                buttonTextContent = "Ok",
                callback = {
                    navController.navigate(DashboardRoute) {
                        popUpTo(navController.currentDestination?.id ?: return@navigate) {
                            inclusive = true
                        }
                    }
                },
            )
        }
    }
}
