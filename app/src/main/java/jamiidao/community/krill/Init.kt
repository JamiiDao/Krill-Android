package jamiidao.community.krill

import android.app.Application
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import jamiidao.community.krill.components.AppText
import jamiidao.community.krill.components.GlassButton
import jamiidao.community.krill.components.KrillLogo
import jamiidao.community.krill.ui.theme.CadmiumOrange
import kotlinx.coroutines.runBlocking

class KrillApplication : Application() {

    @Volatile
    var initResult: Result<Unit>? = null
        private set

    override fun onCreate() {
        app_log("APPLICATION START")

        super.onCreate()

        try {
            runBlocking {
                rustFnInitDb(filesDir.absolutePath)
            }

            initResult = Result.success(Unit)

        } catch (e: Throwable) {
            initResult = Result.failure(e)
        }
    }
}

@Composable
fun InitApp(
    mainActivity: MainActivity
) {
    val app =
        LocalContext.current.applicationContext as KrillApplication

    val initResult = app.initResult

    when {
        initResult == null -> {
            // should never happen if runBlocking completed
        }

        initResult.isSuccess -> {
            AppNavigation(mainActivity)
        }

        else -> {
            when (val error = initResult.exceptionOrNull()) {
                is RustFfiException.AppStorageAlreadyInitialized -> {
                    AppNavigation(mainActivity)
                }

                is RustFfiException -> {
                    ShowAppError(error, mainActivity)
                }
            }
        }
    }
}

@Composable
fun ShowAppError(error: RustFfiException, activity: MainActivity) {

    Column(
        verticalArrangement = Arrangement.SpaceAround,
        horizontalAlignment = Alignment.CenterHorizontally,
        modifier = Modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState())
            .padding(20.dp)
    ) {
        Column(
            verticalArrangement = Arrangement.Center,
            horizontalAlignment = Alignment.CenterHorizontally,
            modifier = Modifier
                .fillMaxSize()
                .padding(20.dp)
        ) {
            KrillLogo()
            Spacer(Modifier.height(50.dp))
            AppText(
                textContent = "App Error",
                fontSize = 30.sp,
                color = CadmiumOrange
            )
            Spacer(Modifier.height(50.dp))
            AppText(
                textContent = "${error.uiMessage()}. Exiting the app and launching the app again might help! If you have already done that file an issue with the team!",
                fontSize = 20.sp
            )
        }

        Column(
            verticalArrangement = Arrangement.Center,
            horizontalAlignment = Alignment.CenterHorizontally,
            modifier = Modifier
                .fillMaxSize()
                .padding(20.dp)
        ) { GlassButton(callback = { activity.finish() }, textContent = "Exit App") }
    }

}

