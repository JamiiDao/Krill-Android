package jamiidao.community.krill

import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.navigation.NavType
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.currentBackStackEntryAsState
import androidx.navigation.compose.rememberNavController
import androidx.navigation.navArgument
import androidx.navigation.navDeepLink
import androidx.navigation.toRoute
import kotlinx.serialization.Serializable

@Serializable
object HomeRoute

@Serializable
object SettingsRoute

@Serializable
data class DeepLinkedRoute(
    val arg: String
)

@Serializable
object NewsRoute

@Serializable
object SecurityRoute

@Serializable
object UpdatesRoute

@Composable
fun AppNavigation(
    paddingValues: PaddingValues
) {
    val navController = rememberNavController()

    val navBackStackEntry by navController.currentBackStackEntryAsState()

    NavHost(
        navController = navController,
        startDestination = HomeRoute,
        modifier = Modifier.padding(paddingValues)
    ) {

        composable<HomeRoute>(

        ) {
            Home()
        }

        composable(
            route = "{arguments}",
            deepLinks = listOf(
                navDeepLink {
                    uriPattern = "krill://{arguments}"
                }
            ),
            arguments = listOf(
                navArgument("arguments") {
                    type = NavType.StringType
                    defaultValue = "not_found"
                }
            )
        ) { backStackEntry ->
            val argumentData = backStackEntry.arguments?.getString("arguments")
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
fun Home() {
    Text(
        text = "Hello ${rustffiFfiVersion()}!",
    )
}

@Composable
fun DeepLinked(data: String?) {
    Text(
        text = "Deeplinked-> $data",
    )
}


@Composable
fun SettingsScreen() {
    Text(
        text = "Settings!",
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