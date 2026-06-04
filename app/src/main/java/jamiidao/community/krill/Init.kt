package jamiidao.community.krill

import android.app.Application
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import androidx.lifecycle.viewmodel.compose.viewModel
import com.google.firebase.messaging.FirebaseMessaging
import jamiidao.community.krill.components.AppText
import jamiidao.community.krill.components.GlassButton
import jamiidao.community.krill.components.KrillLogo
import jamiidao.community.krill.components.KrillStripedLoader
import jamiidao.community.krill.components.ShowErrorAsBottomSheet
import jamiidao.community.krill.ui.theme.CadmiumOrange
import jamiidao.community.krill.ui.theme.bungeeHairlineFamily
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.tasks.await

class AppStateViewModel(application: Application) : AndroidViewModel(application) {

    private val appStorageDir = getApplication<Application>()
        .filesDir.absolutePath
    private val _appState = MutableStateFlow<Result<Unit>?>(null)
    val appState: StateFlow<Result<Unit>?> = _appState

    init {
        viewModelScope.launch {
            _appState.value = initNative(appStorageDir)
        }
    }

    private suspend fun initNative(appStorageDir: String): Result<Unit> {
        return try {
            rustFnInitDb(appStorageDir)
            Result.success(Unit)
        } catch (e: RustFfiException) {
            Result.failure(e)
        }
    }
}


@Composable
fun InitApp(
    mainActivity: MainActivity,
    appStateViewModel: AppStateViewModel = viewModel(),
    paddingValues: PaddingValues
) {
    val initResult by appStateViewModel.appState.collectAsState() // Result<Unit>?

    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(paddingValues = paddingValues)
    ) {
        when {
            initResult == null -> {
                Column(
                    verticalArrangement = Arrangement.SpaceAround,
                    horizontalAlignment = Alignment.CenterHorizontally,
                    modifier = Modifier.fillMaxSize()
                ) {
                    Column(
                        verticalArrangement = Arrangement.Center,
                        horizontalAlignment = Alignment.CenterHorizontally,
                        modifier = Modifier.weight(1f)
                    ) {
                        KrillLogo()
                        Spacer(Modifier.height(10.dp))
                    }
                    Column(
                        verticalArrangement = Arrangement.Top,
                        horizontalAlignment = Alignment.CenterHorizontally,
                        modifier = Modifier.weight(1f)
                    ) {
                        AppText(
                            textContent = "INITIALIZING APP",
                            fontSize = 25.sp,
                            fontWeight = FontWeight.Bold,
                            fontFamily = bungeeHairlineFamily
                        )
                        Spacer(Modifier.height(10.dp))
                        KrillStripedLoader()
                    }
                }
            }

            initResult?.isSuccess == true -> {
                AppNavigation(mainActivity)
            }

            else -> {
                when (val error = initResult?.exceptionOrNull()) {
                    is RustFfiException.AppStorageAlreadyInitialized -> {
                        AppNavigation(mainActivity)
                    }

                    else -> {
                        ShowAppError(error as RustFfiException, mainActivity)
                    }
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

