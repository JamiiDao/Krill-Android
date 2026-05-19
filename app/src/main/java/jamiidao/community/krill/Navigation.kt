package jamiidao.community.krill

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.blur
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import androidx.navigation.NavType
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.currentBackStackEntryAsState
import androidx.navigation.compose.rememberNavController
import androidx.navigation.navArgument
import androidx.navigation.navDeepLink
import jamiidao.community.krill.components.KrillBorderRing
import jamiidao.community.krill.components.KrillGlassSurface
import jamiidao.community.krill.components.KrillStripedLoader
import jamiidao.community.krill.dashboard.DashboardShell
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

    NavHost(
        navController = navController,
        startDestination = HomeRoute,
        modifier = Modifier.padding(paddingValues)
    ) {

        composable<HomeRoute> {
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

    DashboardShell(
        content = {
            Column(
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.SpaceAround,
                modifier = Modifier.fillMaxSize()
            ) {

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
                                        text = "Hello ${rustffiFfiVersion()}!",
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
                                text = "Hello ${rustffiFfiVersion()}!",
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

