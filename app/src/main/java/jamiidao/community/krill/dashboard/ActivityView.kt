package jamiidao.community.krill.dashboard

import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.navigation.NavController
import jamiidao.community.krill.DashboardRoute
import jamiidao.community.krill.ErrorRoute
import jamiidao.community.krill.R
import jamiidao.community.krill.RustFfiException
import jamiidao.community.krill.RustTypeActivityMetadata
import jamiidao.community.krill.ViewGroupActivitiesRoute
import jamiidao.community.krill.app_log
import jamiidao.community.krill.components.AppText
import jamiidao.community.krill.components.GlassButton
import jamiidao.community.krill.components.GlassButtonWithComposable
import jamiidao.community.krill.components.KrillGlassSurface
import jamiidao.community.krill.components.KrillStripedLoader
import jamiidao.community.krill.components.ShowErrorAsNormalView
import jamiidao.community.krill.getTimezoneOffset
import jamiidao.community.krill.rustFnGetActivity
import jamiidao.community.krill.rustFnParseActivityDeeplink
import jamiidao.community.krill.rustFnParticipateInActivity
import jamiidao.community.krill.ui.theme.CadmiumOrange
import jamiidao.community.krill.ui.theme.bungeeHairlineFamily
import jamiidao.community.krill.ui.theme.commitMonoFamily
import kotlinx.coroutines.launch


@Composable
fun ActivityMetadata(navController: NavController, data: String) {
    val timezoneOffset = getTimezoneOffset()
    val fetchedActivityMetadata =
        remember { mutableStateOf<Result<RustTypeActivityMetadata?>?>(null) }

    Column(
        verticalArrangement = Arrangement.Center,
        horizontalAlignment = Alignment.CenterHorizontally,
        modifier = Modifier
            .padding(12.dp)
            .fillMaxSize()
    ) {

        runCatching {
            rustFnParseActivityDeeplink(data)
        }.fold(
            onSuccess = { deeplink ->
                LaunchedEffect(Unit) {
                    fetchedActivityMetadata.value =
                        runCatching {
                            val foo = rustFnGetActivity(deeplink, timezoneOffset)
                            app_log("$foo");

                            foo
                        }
                }
                fetchedActivityMetadata.value?.let {
                    it.onSuccess { metadata ->
                        SuccessView(navController, metadata)
                    }
                        .onFailure { e ->

                            if (e is RustFfiException) {
                                ShowErrorAsNormalView(
                                    navController,
                                    title = "Invalid Deeplink",
                                    error = e.uiMessage()
                                )
                            } else {
                                ShowErrorAsNormalView(
                                    navController,
                                    title = "Invalid Deeplink",
                                    error = e.message ?: "Unable to check the deeplink"
                                )
                            }
                        }
                } ?: Column(
                    verticalArrangement = Arrangement.Center,
                    horizontalAlignment = Alignment.CenterHorizontally,
                    modifier = Modifier.fillMaxSize()
                ) {
                    AppText(textContent = deeplink.domain, fontSize = 18.sp, color = CadmiumOrange)
                    Spacer(Modifier.height(5.dp))
                    KrillStripedLoader()
                }
            },
            onFailure = { e ->
                if (e is RustFfiException) {
                    ShowErrorAsNormalView(
                        navController,
                        title = "Invalid Deeplink",
                        error = e.uiMessage()
                    )
                } else {
                    ShowErrorAsNormalView(
                        navController,
                        error = e.message ?: "Unable to check the deeplink"
                    )
                }
            }
        )
    }
}

@Composable
fun SuccessView(navController: NavController, metadata: RustTypeActivityMetadata?) {
    val scope = rememberCoroutineScope()

    val buttonEnabled = remember { mutableStateOf(true) }

    metadata?.let {
        Column(
            verticalArrangement = Arrangement.SpaceBetween,
            horizontalAlignment = Alignment.CenterHorizontally,
            modifier = Modifier
                .fillMaxSize()
                .padding(8.dp)
        ) {
            Column(
                verticalArrangement = Arrangement.Center,
                horizontalAlignment = Alignment.CenterHorizontally,
                modifier = Modifier
                    .weight(.8f)
                    .fillMaxWidth()
            ) {
                KrillGlassSurface(
                    modifier = Modifier.fillMaxWidth(),
                    percentage = 5,
                    content = {
                        Column(
                            verticalArrangement = Arrangement.Top,
                            horizontalAlignment = Alignment.CenterHorizontally,
                            modifier = Modifier
                                .fillMaxWidth()
                                .padding(8.dp)
                        ) {
                            AppText(
                                textContent = it.name,
                                fontFamily = bungeeHairlineFamily,
                                fontSize = 25.sp,
                                fontWeight = FontWeight.Black,
                                color = CadmiumOrange
                            )

                            Spacer(Modifier.height(30.dp))

                            SuccessViewRow(
                                metadata.creator,
                                { IconImage(R.drawable.avatar_filled) })

                            SuccessViewRow("${metadata.spend} SOL", {
                                AppText(
                                    textContent = "◎",
                                    color = CadmiumOrange
                                )
                            })

                            SuccessViewRow(
                                "${metadata.threshold.min} minimum signers",
                                { IconImage(R.drawable.signature) })
                            SuccessViewRow(
                                "${metadata.threshold.max} maximum signers",
                                { IconImage(R.drawable.signature) })

                            Spacer(Modifier.height(5.dp))

                            SuccessViewRow(
                                metadata.timestamp,
                                { IconImage(R.drawable.calendar) }, 14.sp
                            )
                        }
                    })
            }

            Column(
                verticalArrangement = if (buttonEnabled.value) {
                    Arrangement.Bottom
                } else {
                    Arrangement.Center
                },
                horizontalAlignment = Alignment.CenterHorizontally,
                modifier = Modifier
                    .weight(.8f)
                    .fillMaxWidth()
            ) {
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.Absolute.Center,
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(8.dp)
                ) {
                    if (buttonEnabled.value) {
                        GlassButton(
                            textContent = "Skip",
                            callback = {
                                navController.navigate(DashboardRoute)
                            },
                            width = .4f,
                            filled = false
                        )
                    }

                    GlassButtonWithComposable(
                        content = {
                            Text(
                                color = Color.White,
                                text = "Participate",
                            )
                        },
                        callback = {
                            buttonEnabled.value = false

                            scope.launch {
                                try {
                                    val domainOrIp = rustFnParticipateInActivity(it)
                                    navController.navigate(
                                        ViewGroupActivitiesRoute(
                                            domainOrIp,
                                        )
                                    ) {
                                        popUpTo(DashboardRoute)
                                    }
                                } catch (e: RustFfiException) {
                                    navController.navigate(
                                        ErrorRoute(
                                            title = "Unable to join activity",
                                            error = e.uiMessage()
                                        )
                                    ) {
                                        popUpTo(DashboardRoute)
                                    }
                                }
                            }
                        },
                        width = 1f,
                        filled = !buttonEnabled.value,
                        buttonEnabled

                    )
                }
            }
        }
    } ?: Column {
        ShowErrorAsNormalView(
            navController,
            title = "Deeplink Error",
            error = "Activity Not Found!"
        )
    }
}

@Composable
fun IconImage(target: Int) {
    Image(
        painter = painterResource(id = target),
        contentDescription = "",
        colorFilter = ColorFilter.tint(CadmiumOrange),
        modifier = Modifier.size(20.dp)
    )
}

@Composable
fun SuccessViewRow(contentText: String, icon: @Composable () -> Unit, fontSize: TextUnit = 18.sp) {
    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.Start,
        modifier = Modifier.fillMaxWidth()
    ) {
        Box(
            contentAlignment = Alignment.Center,
            modifier = Modifier.weight(.2f)
        ) {
            icon()
        }
        Box(
            contentAlignment = Alignment.CenterStart,
            modifier = Modifier.weight(.9f)
        ) {
            AppText(
                textContent = contentText,
                fontFamily = commitMonoFamily,
                fontSize = fontSize
            )
        }
    }
}
