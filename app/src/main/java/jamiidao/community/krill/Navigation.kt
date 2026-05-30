package jamiidao.community.krill

import android.content.Intent
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import androidx.navigation.NavType
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import androidx.navigation.navArgument
import androidx.navigation.navDeepLink
import jamiidao.community.krill.components.KrillBorderRing
import jamiidao.community.krill.components.KrillGlassSurface
import jamiidao.community.krill.components.KrillLogo
import jamiidao.community.krill.components.KrillStripedLoader
import jamiidao.community.krill.dashboard.DashboardShell
import jamiidao.community.krill.notifications_module.RequestNotificationPermissionScreen
import jamiidao.community.krill.notifications_module.hasNotificationPermission
import jamiidao.community.krill.notifications_module.needsNotificationPermission
import kotlinx.coroutines.launch
import kotlinx.serialization.Serializable

@Serializable
object DashboardRoute

@Serializable
object RequestNotificationPermissionRoute


@Serializable
object NewsRoute

@Serializable
object SecurityRoute

@Serializable
object UpdatesRoute

@Composable
fun AppNavigation(
    activity: MainActivity, appStateViewModel: AppStateViewModel,
    paddingValues: PaddingValues
) {
    val navController = rememberNavController()

    LaunchedEffect(Unit) {
        try {
            app_log("Navigation: HELLO QUIC Success")

        } catch (e: RustFfiException) {
            app_log("Navigation: HELLO QUIC: ${e.uiMessage()}")
        }
    }

    NavHost(
        navController = navController,
        startDestination = DashboardRoute,
        modifier = Modifier.padding(paddingValues)
    ) {

        composable<RequestNotificationPermissionRoute> {
            if (needsNotificationPermission()) {
                RequestNotificationPermissionScreen(
                    activity,
                    navController,
                )
            } else {
                DashboardView()
            }
        }

        composable<DashboardRoute> {
            DashboardView()
        }

        composable(
            route = "{action}/{arguments}",
            deepLinks = listOf(navDeepLink {
                uriPattern = "krill://{action}/{arguments}"
                action = Intent.ACTION_VIEW
            }),
            arguments = listOf(
                navArgument("join") {
                    type = NavType.StringType
                    defaultValue = "default"
                },
                navArgument("arguments") {
                    type = NavType.StringType
                    defaultValue = ""
                }
            )
        ) { backStackEntry ->
            val action = backStackEntry.arguments?.getString("action")
            val argumentData = backStackEntry.arguments?.getString("arguments")

            app_log("NOTIFICATION_DEEPLINK: $action/$argumentData")

            DeepLinked(argumentData)
        }

        composable<NewsRoute>(
            deepLinks = listOf(
                navDeepLink<NewsRoute>(
                    basePath = "https://krill.pro/news"
                )
            )
        ) {
            NewsScreen()
        }

        composable<SecurityRoute>(
            deepLinks = listOf(
                navDeepLink<SecurityRoute>(
                    basePath = "https://krill.pro/security"
                )
            )
        ) {
            SecurityScreen()
        }

        composable<UpdatesRoute>(
            deepLinks = listOf(
                navDeepLink<UpdatesRoute>(
                    basePath = "https://krill.pro/updates"
                )
            )
        ) {
            UpdatesScreen()
        }
    }
}


@Composable
fun DashboardView() {
    val selectedRoute = if (hasNotificationPermission(LocalContext.current)) {
        DashboardRoute
    } else {
        RequestNotificationPermissionRoute
    }

    DashboardShell(
        content = {
            Column(
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.SpaceAround,
                modifier = Modifier.fillMaxSize()
            ) {
                KrillLogo()

                Box(
                    modifier = Modifier.fillMaxSize(.8f)
                ) {
                    KrillGlassSurface(percentage = 5) {
                        Column(
                            modifier = Modifier.height(100.dp)
                        ) {
                            Text("VISIBLE TEST")

                            KrillGlassSurface(
                                content = {
                                    Text(
                                        color = Color.White,
                                        text = "Hello ${rustFnFfiVersion()}!",
                                    )
                                })
                        }
                    }
                }

                KrillStripedLoader()

                KrillBorderRing(
                    borderColor = Color(0xFFFF6600),
                    shape = RoundedCornerShape(percent = 50)
                ) {
                    KrillGlassSurface(
                        content = {
                            Text(
                                color = Color.White,
                                text = "Hello ${rustFnFfiVersion()}!",
                            )
                        })
                }
            }

        }
    )
}


@Composable
fun DeepLinked(data: String?) {
    Text(
        text = "Deeplinked-> $data",
    )
}

@Composable
fun NewsScreen() {
    Text("News")
}

@Composable
fun SecurityScreen() {
    Text("Security")
}

@Composable
fun UpdatesScreen() {
    Text("Updates")
}

