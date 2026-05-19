package jamiidao.community.krill.notifications_module

import android.Manifest
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.os.Build
import android.provider.Settings
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.annotation.RequiresApi
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.core.app.ActivityCompat.shouldShowRequestPermissionRationale
import androidx.core.content.ContextCompat
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleEventObserver
import androidx.lifecycle.compose.LocalLifecycleOwner
import androidx.navigation.NavController
import jamiidao.community.krill.DashboardRoute
import jamiidao.community.krill.MainActivity
import jamiidao.community.krill.RustTypeAppPermissionState
import jamiidao.community.krill.components.AppText
import jamiidao.community.krill.components.GlassButton
import jamiidao.community.krill.components.KrillLogo

@RequiresApi(Build.VERSION_CODES.TIRAMISU)
@Composable
fun RequestNotificationPermissionScreen(
    activity: MainActivity,
    navController: NavController,
) {
    val state = currentNotificationPermState(activity)
    val notificationPermState =
        remember { mutableStateOf(state) }

    val context = LocalContext.current

    val permanentlyDeniedText =
        "You have permanently denied notification permission. You can enable it from this app's settings!"

    val info = remember {
        mutableStateOf(
            "Give this app the permission to receive notifications " +
                    if (needsPromotedNotificationPermission()) {
                        "and live updates "
                    } else {
                        " "
                    } +
                    "for group events and other time-sensitive information."
        )
    }

    val permission: String =
        Manifest.permission.POST_NOTIFICATIONS

    val permissionLauncher = rememberLauncherForActivityResult(
        contract = ActivityResultContracts.RequestPermission(),
        onResult = { isGranted ->
            val state = if (!isGranted) {
                if (shouldShowRequestPermissionRationale(
                        activity,
                        permission
                    )
                ) {
                    RustTypeAppPermissionState.DENIED
                } else {
                    RustTypeAppPermissionState.PERMANENTLY_DENIED
                }
            } else {
                RustTypeAppPermissionState.GRANTED
            }

            notificationPermState.value = state
        }
    )

    val lifecycleOwner = LocalLifecycleOwner.current

    DisposableEffect(lifecycleOwner) {
        val observer = LifecycleEventObserver { _, event ->
            if (event == Lifecycle.Event.ON_RESUME) {
                if (hasNotificationPermission(context)) {
                    navController.navigate(DashboardRoute)
                }
            }
        }

        lifecycleOwner.lifecycle.addObserver(observer)
        onDispose {
            lifecycleOwner.lifecycle.removeObserver(observer)
        }
    }

    Column(
        modifier = Modifier
            .fillMaxSize(), verticalArrangement = Arrangement.SpaceAround,
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
                textContent = info.value,
                fontSize = 20.sp,
                textAlign = TextAlign.Center
            )
        }

        when (notificationPermState.value) {
            RustTypeAppPermissionState.GRANTED -> navController.navigate(DashboardRoute)
            RustTypeAppPermissionState.DENIED -> {
                Row(
                    modifier = Modifier
                        .weight(.4f)
                        .fillMaxWidth(),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.SpaceBetween
                ) {
                    GlassButton(
                        textContent = "Skip",
                        callback = {
                            navController.navigate(DashboardRoute) {
                                popUpTo(DashboardRoute) { inclusive = true }
                            }
                        },
                        width = .3f,
                    )
                    GlassButton(
                        textContent = "Allow Notifications",
                        callback = {
                            permissionLauncher.launch(Manifest.permission.POST_NOTIFICATIONS)
                        },
                        width = .9f,
                        filled = true
                    )
                }
            }

            RustTypeAppPermissionState.PERMANENTLY_DENIED -> {
                info.value = permanentlyDeniedText

                Row(
                    modifier = Modifier
                        .weight(.4f)
                        .fillMaxWidth(),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.Center
                ) {
                    GlassButton(
                        textContent = "Access App Settings",
                        callback = {
                            val intent = Intent().apply {
                                action = Settings.ACTION_APP_NOTIFICATION_SETTINGS
                                putExtra(Settings.EXTRA_APP_PACKAGE, context.packageName)
                            }
                            context.startActivity(intent)
                        },
                        width = .9f,
                        filled = true,
                    )
                }
            }
        }
    }
}

fun needsPromotedNotificationPermission(): Boolean {
    return Build.VERSION.SDK_INT >= Build.VERSION_CODES.BAKLAVA
}

fun needsNotificationPermission(): Boolean {
    return Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU
}

fun hasNotificationPermission(context: Context): Boolean {
    return if (needsNotificationPermission()) {
        ContextCompat.checkSelfPermission(
            context,
            Manifest.permission.POST_NOTIFICATIONS
        ) == PackageManager.PERMISSION_GRANTED
    } else {
        true
    }
}


@Composable
fun currentNotificationPermState(
    activity: MainActivity,
): RustTypeAppPermissionState {
    if (!needsNotificationPermission()) {
        return RustTypeAppPermissionState.GRANTED
    }

    val permission: String =
        Manifest.permission.POST_NOTIFICATIONS

    val notificationState = remember { mutableStateOf(RustTypeAppPermissionState.DENIED) }

    rememberLauncherForActivityResult(
        contract = ActivityResultContracts.RequestPermission(),
        onResult = { isGranted ->
            val state = if (!isGranted) {
                if (shouldShowRequestPermissionRationale(
                        activity,
                        permission
                    )
                ) {
                    RustTypeAppPermissionState.DENIED
                } else {
                    RustTypeAppPermissionState.PERMANENTLY_DENIED
                }
            } else {
                RustTypeAppPermissionState.GRANTED
            }

            notificationState.value = state
        }
    )

    return notificationState.value
}