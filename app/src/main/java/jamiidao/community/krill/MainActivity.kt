package jamiidao.community.krill

import android.content.Intent
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.viewModels
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.Scaffold
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.lifecycle.lifecycleScope
import com.google.firebase.messaging.FirebaseMessaging
import jamiidao.community.krill.components.KrillDotsBackground
import jamiidao.community.krill.components.ShowErrorAsBottomSheet
import jamiidao.community.krill.notifications_module.createSigningNotificationChannel
import jamiidao.community.krill.ui.theme.KrillTheme
import kotlinx.coroutines.launch
import kotlinx.coroutines.tasks.await

class MainActivity : ComponentActivity() {
    init {
        System.loadLibrary("krill_native")
    }

    val rootActivity = this
    private val appStateViewModel: AppStateViewModel by viewModels()


    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        createSigningNotificationChannel(logExisting = true)

        enableEdgeToEdge()

        lifecycleScope.launch {
            try {
                val token = FirebaseMessaging.getInstance().token.await()
                rustFnSetFcmToken(token)
            } catch (e: Exception) {
                app_log("Failed to get FCM token: ${e.message}")
            }
        }

        setContent {
            KrillTheme {
                Scaffold(modifier = Modifier.fillMaxSize()) { innerPadding ->
                    Box(
                        contentAlignment = Alignment.Center,
                        modifier = Modifier
                            .fillMaxSize()
                    ) {
                        // Background layer
                        KrillDotsBackground(
                            modifier = Modifier
                                .fillMaxSize()
                        )
                        InitApp(mainActivity = rootActivity, appStateViewModel, innerPadding)
                    }
                }
            }
        }
    }

    // Called after onCreate() or when returning from background
    override fun onStart() {
        super.onStart()
    }

    // Called after onStart(), activity is now interactive
    override fun onResume() {
        super.onResume()
    }

    // Called when activity is partially obscured (e.g., new activity is pushed on top)
    override fun onPause() {
        super.onPause()
    }

    // Called when activity is no longer visible
    override fun onStop() {
        super.onStop()
    }

    // Called when activity is being restarted after being stopped
    override fun onRestart() {
        super.onRestart()
    }

    // Called before the activity is destroyed (e.g., back press, system kill)
    override fun onDestroy() {
        super.onDestroy()
    }

    // Called when the system is restoring state after process death
    override fun onRestoreInstanceState(savedInstanceState: Bundle) {
        super.onRestoreInstanceState(savedInstanceState)
    }

    // Called when the system is saving state (e.g., before killing the app)
    override fun onSaveInstanceState(outState: Bundle) {
        super.onSaveInstanceState(outState)
    }
}
