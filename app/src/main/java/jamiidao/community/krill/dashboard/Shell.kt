package jamiidao.community.krill.dashboard

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material3.Badge
import androidx.compose.material3.BadgedBox
import androidx.compose.material3.Icon
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.NavigationBarItemDefaults
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.res.vectorResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import jamiidao.community.krill.R
import jamiidao.community.krill.components.AppText
import jamiidao.community.krill.components.KrillGlassSurface
import jamiidao.community.krill.ui.theme.CadmiumOrange
import jamiidao.community.krill.ui.theme.White


const val ROUTE_GROUPS = "Groups"
const val ROUTE_SCAN_QR = "Scan"
const val ROUTE_PROFILE = "Profile"

data class BottomNavigationItem(
    val title: String,
    val itemIcon: ImageVector,
    val hasNews: Boolean,
    val badgeCount: Int? = null
)

@Composable
fun DashboardShell(
    content: @Composable () -> Unit
) {


    val dashboardNavController = rememberNavController()

    val items = listOf(
        BottomNavigationItem(
            title = ROUTE_GROUPS,
            itemIcon = ImageVector.vectorResource(R.drawable.members),
            hasNews = false,
        ),
        BottomNavigationItem(
            title = ROUTE_SCAN_QR,
            itemIcon = ImageVector.vectorResource(R.drawable.qrcode),
            hasNews = false,
        ),
        BottomNavigationItem(
            title = ROUTE_PROFILE,
            itemIcon = ImageVector.vectorResource(R.drawable.avatar),
            hasNews = false,
        ),
    )

    var selectedItemIndex by rememberSaveable { mutableIntStateOf(0) }

    val haptic = LocalHapticFeedback.current

    Scaffold(
        containerColor = Color.Transparent,
        bottomBar = {
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .background(Color.Transparent)
                    .padding(
                        horizontal = 4.dp,
                    ),
            ) {
                NavigationBar(
                    modifier = Modifier
                        .fillMaxWidth(),
                    containerColor = Color.Transparent,
                    windowInsets = WindowInsets(0)
                ) {
                    items.forEachIndexed { index, item ->
                        NavigationBarItem(
                            selected = selectedItemIndex == index,
                            onClick = {
                                haptic.performHapticFeedback(HapticFeedbackType.ContextClick)
                                selectedItemIndex = index
                                dashboardNavController.navigate(item.title)
                            },
                            icon = {
                                BadgedBox(
                                    modifier = Modifier
                                        .fillMaxWidth()
                                        .padding(0.dp),
                                    badge = {
                                        if (item.badgeCount != null) {
                                            Badge {
                                                Text("ADD ICON")
                                            }
                                        } else if (item.hasNews) {
                                            Badge()
                                        }
                                    }) {
                                    KrillGlassSurface(
                                        show = (selectedItemIndex == index),
                                        horizontalPadding = 10.dp,
                                        verticalPadding = 6.dp
                                    ) {
                                        Row(
                                            horizontalArrangement = Arrangement.Center,
                                            verticalAlignment = Alignment.CenterVertically,
                                            modifier = Modifier
                                                .fillMaxWidth(),
                                        ) {
                                            Icon(
                                                imageVector = item.itemIcon,
                                                contentDescription = item.title,
                                                modifier = Modifier.size(30.dp),
                                                tint = if (selectedItemIndex == index) {
                                                    CadmiumOrange
                                                } else {
                                                    White
                                                }
                                            )
                                            if (selectedItemIndex != index) {
                                                Spacer(Modifier.width(5.dp))
                                            }
                                            AppText(
                                                textContent = item.title,
                                                fontSize = 16.sp,
                                                color = if (selectedItemIndex == index) {
                                                    CadmiumOrange
                                                } else {
                                                    White
                                                },
                                                fontWeight = if (selectedItemIndex == index) {
                                                    FontWeight.SemiBold
                                                } else {
                                                    FontWeight.Bold
                                                }
                                            )
                                        }
                                    }
                                }
                            },
                            colors = NavigationBarItemDefaults.colors(
                                indicatorColor = Color.Transparent
                            )
                        )
                    }
                }
            }
        }
    ) { padding ->

        Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.Top,
            modifier = Modifier
                .fillMaxSize()
                .padding(padding)
                .background(Color.Transparent)
        ) {

            content()

            NavHost(
                navController = dashboardNavController,
                startDestination = ROUTE_GROUPS,
                modifier = Modifier.fillMaxSize()
            ) {
                composable(ROUTE_GROUPS) { }

                composable(ROUTE_SCAN_QR) {

                }
                composable(ROUTE_PROFILE) { }
            }
        }
    }
}