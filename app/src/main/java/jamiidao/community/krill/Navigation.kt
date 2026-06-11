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
import androidx.navigation.NavController
import androidx.navigation.NavType
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import androidx.navigation.navArgument
import androidx.navigation.navDeepLink
import androidx.navigation.toRoute
import com.google.firebase.messaging.FirebaseMessaging
import jamiidao.community.krill.components.KrillBorderRing
import jamiidao.community.krill.components.KrillGlassSurface
import jamiidao.community.krill.components.KrillLogo
import jamiidao.community.krill.components.KrillStripedLoader
import jamiidao.community.krill.components.ShowErrorAsNormalView
import jamiidao.community.krill.dashboard.ActivityMetadata
import jamiidao.community.krill.dashboard.DashboardShell
import jamiidao.community.krill.dashboard.SuccessView
import jamiidao.community.krill.dashboard.ViewOrganizationView
import jamiidao.community.krill.deeplinks.JoinOrganization
import jamiidao.community.krill.notifications_module.RequestNotificationPermissionScreen
import jamiidao.community.krill.notifications_module.hasNotificationPermission
import jamiidao.community.krill.notifications_module.needsNotificationPermission
import kotlinx.coroutines.launch
import kotlinx.coroutines.tasks.await
import kotlinx.serialization.Serializable
import java.time.ZoneId
import java.time.ZonedDateTime

@Serializable
object DashboardRoute

@Serializable
object RequestNotificationPermissionRoute

@Serializable
data class ViewGroupActivitiesRoute(
    val sldTld: String,
)

@Serializable
object NewsRoute

@Serializable
object SecurityRoute

@Serializable
object UpdatesRoute

@Serializable
data class ErrorRoute(
    val title: String = "Encountered Error",
    val error: String,
    val imageID: Int = R.drawable.error,
    val imageDescription: String = "",
    val buttonTextContent: String = "Ok",
    val glassSurface: Boolean = true
)


@Composable
fun AppNavigation(
    mainActivity: MainActivity,
) {
    val navController = rememberNavController()
    val appDirPath = appStoragePath(LocalContext.current)

    LaunchedEffect(Unit) {
        try {
            val token = FirebaseMessaging.getInstance().token.await()
            rustFnSetFcmToken(appDirPath, token)
            app_log("Received FCM token")
        } catch (e: RustFfiException) {
            app_log("Failed to send FCM token: ${e.uiMessage()}")

        }

    }

    NavHost(
        navController = navController,
        startDestination = DashboardRoute,
    ) {
        composable<RequestNotificationPermissionRoute> {
            if (needsNotificationPermission()) {
                RequestNotificationPermissionScreen(
                    mainActivity,
                    navController,
                )
            } else {
                DashboardView(mainActivity, navController)
            }
        }

        composable<DashboardRoute> {
            DashboardView(mainActivity, navController)
        }

        composable<ViewGroupActivitiesRoute> { backStackEntry ->
            val routeData: ViewGroupActivitiesRoute = backStackEntry.toRoute()
            ViewOrganizationView(navController, routeData.sldTld)
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

            argumentData?.let {
                when (action) {
                    "join" -> {
                        JoinOrganization(navController, argumentData)
                    }

                    "dkg" -> {
                        ActivityMetadata(navController, argumentData)
                    }

                    else -> {
                        LaunchedEffect(Unit) {
                            navController.popBackStack()
                        }
                        return@composable
                    }
                }
            }
                ?: LaunchedEffect(Unit) {
                    navController.popBackStack()
                }
            return@composable
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

        composable<ErrorRoute> { backStackEntry ->
            val routeData: ErrorRoute = backStackEntry.toRoute()
            ShowErrorAsNormalView(
                navController,
                title = routeData.title,
                error = routeData.error,
            )
        }
    }
}


@Composable
fun DashboardView(mainActivity: MainActivity, navController: NavController) {
    if (needsNotificationPermission() && !hasNotificationPermission(mainActivity)) {
        RequestNotificationPermissionScreen(
            mainActivity,
            navController,
        )
    } else {
        DashboardShell(
            mainActivity, navController,
        )
    }
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
