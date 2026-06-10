package jamiidao.community.krill

import android.content.Intent
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.viewModels
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import jamiidao.community.krill.components.AppText
import jamiidao.community.krill.components.GlassButton
import jamiidao.community.krill.components.KrillDotsBackground
import jamiidao.community.krill.frostServices.FrostDkgHandler
import jamiidao.community.krill.notifications_module.createSigningNotificationChannel
import jamiidao.community.krill.ui.theme.KrillTheme
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.launch

class MainActivity : ComponentActivity() {
    init {
        System.loadLibrary("krill_native")
    }

    val rootActivity = this

    override fun onCreate(savedInstanceState: Bundle?) {

        app_log("ACTIVITY START")


        super.onCreate(savedInstanceState)

        createSigningNotificationChannel(logExisting = true)

        enableEdgeToEdge()

        setContent {
            KrillTheme {
                Scaffold(modifier = Modifier.fillMaxSize()) { innerPadding ->
                    innerPadding;
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
                        InitApp(mainActivity = rootActivity)
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


