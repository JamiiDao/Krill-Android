package jamiidao.community.krill.dashboard

import android.content.Intent
import android.view.WindowInsets
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeContent
import androidx.compose.foundation.layout.safeContentPadding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.navigation.NavController
import coil.ImageLoader
import coil.compose.AsyncImage
import coil.decode.SvgDecoder
import jamiidao.community.krill.ActivityEmitter
import jamiidao.community.krill.ActivityListener
import jamiidao.community.krill.ActivityListenerOutcome
import jamiidao.community.krill.DashboardRoute
import jamiidao.community.krill.R
import jamiidao.community.krill.RustFfiException
import jamiidao.community.krill.RustTypeActivityMetadata
import jamiidao.community.krill.RustTypeActivitySubscriberChannel
import jamiidao.community.krill.RustTypeMinMax
import jamiidao.community.krill.RustTypeOrganizationInfo
import jamiidao.community.krill.RustTypeStoredOrgInfoMetadata
import jamiidao.community.krill.app_log
import jamiidao.community.krill.components.AppText
import jamiidao.community.krill.components.KrillGlassSurface
import jamiidao.community.krill.components.KrillLogo
import jamiidao.community.krill.components.KrillStripedLoader
import jamiidao.community.krill.components.ShowErrorAsBottomSheet
import jamiidao.community.krill.frostServices.FrostDkgHandler
import jamiidao.community.krill.getTimezoneOffset
import jamiidao.community.krill.rustFnLoadStoredOrganizationInfo
import jamiidao.community.krill.ui.theme.CadmiumOrange
import jamiidao.community.krill.ui.theme.bungeeHairlineFamily
import jamiidao.community.krill.ui.theme.commitMonoFamily
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.launch


@Composable
fun ViewOrganizationView(
    navController: NavController,
    sldTld: String,
) {
    val state = remember { mutableStateOf(ComponentState.Fetching) }
    val orgInfo = remember { mutableStateOf<RustTypeStoredOrgInfoMetadata?>(null) }
    val error = remember { mutableStateOf<String?>(null) }
    val timezoneOffset = getTimezoneOffset()


    LaunchedEffect(Unit) {
        try {
            orgInfo.value = rustFnLoadStoredOrganizationInfo(sldTld, timezoneOffset)

            state.value = ComponentState.Done
        } catch (e: RustFfiException) {
            error.value = e.uiMessage()
        }
    }

    when (state.value) {
        ComponentState.Done -> {
            app_log("FETCHING SINCE DONE")
        }

        else -> {}
    }


    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center,
        modifier = Modifier
            .fillMaxSize()
            .safeContentPadding()
    ) {
        when (state.value) {
            ComponentState.Fetching -> {
                KrillLogo()
                Spacer(Modifier.height(10.dp))
                AppText(
                    textContent = "Loading $sldTld", fontSize = 18.sp,
                    fontWeight = FontWeight.Normal, fontFamily = commitMonoFamily,
                    color = CadmiumOrange
                )
                Spacer(Modifier.height(5.dp))
                KrillStripedLoader()
            }

            ComponentState.Done -> {
                orgInfo.value?.let {
                    OrgDetails(navController, it)
                } ?: Column(
                    horizontalAlignment = Alignment.CenterHorizontally,
                    verticalArrangement = Arrangement.SpaceAround,
                    modifier = Modifier.fillMaxSize()
                ) {
                    AppText(textContent = "Organization Not Found")
                    Spacer(Modifier.height(30.dp))
                }
            }
        }
    }

    error.value?.let {
        ShowErrorAsBottomSheet(
            title = "Get Organization",
            error = it,
            imageID = R.drawable.error,
            imageDescription = "",
            buttonTextContent = "Ok",
            callback = {
                navController.navigate(DashboardRoute) {
                    popUpTo(navController.currentDestination?.id ?: return@navigate) {
                        inclusive = false
                    }
                }
            },
        )
    }
}

@Composable
fun OrgDetails(
    navController: NavController,
    metadata: RustTypeStoredOrgInfoMetadata,
) {
    val sldTld = metadata.sldTld

    val context = LocalContext.current

    val imageLoader = ImageLoader.Builder(context)
        .components {
            add(SvgDecoder.Factory())
        }
        .build()
    Column(
        verticalArrangement = Arrangement.SpaceAround,
        horizontalAlignment = Alignment.CenterHorizontally,
        modifier = Modifier.fillMaxSize()
    ) {
        KrillGlassSurface(
            modifier = Modifier.fillMaxWidth(),
            percentage = 10,
            content = {
                Column(
                    horizontalAlignment = Alignment.CenterHorizontally,
                    verticalArrangement = Arrangement.Center,
                    modifier = Modifier
                ) {
                    AsyncImage(
                        model = metadata.logoIcon,
                        imageLoader = imageLoader,
                        contentDescription = null,
                        modifier = Modifier.size(200.dp)
                    )
                    Spacer(Modifier.height(10.dp))
                    AppText(
                        textContent = metadata.orgName,
                        fontSize = 28.sp,
                        fontFamily = bungeeHairlineFamily,
                        fontWeight = FontWeight.Black
                    )
                    Spacer(Modifier.height(5.dp))
                    AppText(
                        textContent = metadata.sldTld,
                        fontSize = 18.sp,
                        fontFamily = commitMonoFamily,
                        color = CadmiumOrange
                    )
                    Spacer(Modifier.height(20.dp))
                    AppText(
                        textContent = metadata.supportMail,
                        fontSize = 18.sp,
                        fontFamily = commitMonoFamily,
                    )
                    Spacer(Modifier.height(20.dp))
                }
            })

        Spacer(Modifier.height(10.dp))

        metadata.active?.let { currentIt ->
            val dkgHandler = remember { RustDkgHandler(sldTld, currentIt) }

            val dataOutcome by dkgHandler.state.collectAsState()

            val ackReceived = remember { mutableStateOf(false) }
            val collectEvents = remember {
                mutableListOf(
                    "Listening"
                )
            }

            dataOutcome?.let { currentIt ->
                when (val recvData = currentIt.data) {
                    RustTypeActivitySubscriberChannel.ACK -> {
                        LaunchedEffect(Unit) {
                            app_log("STARTED.....")

                            Intent(context, FrostDkgHandler::class.java).also { intent ->
                                intent.action = FrostDkgHandler.ACTION_START
                                context.startForegroundService(intent)
                            }

                            ackReceived.value = true
                        }
                    }

                    RustTypeActivitySubscriberChannel.NEW_SUBSCRIBER -> {
                        collectEvents.add(recvData.toUiMessage())
                    }

                    RustTypeActivitySubscriberChannel.TERMINATED -> {
                        if (ackReceived.value) {
                            LaunchedEffect(Unit) {
                                app_log("STOPPED.....")

                                Intent(context, FrostDkgHandler::class.java).also { intent ->
                                    intent.action = FrostDkgHandler.ACTION_STOP
                                    context.startForegroundService(intent)
                                }
                            }
                        }
                    }
                }

                LazyColumn(
                    verticalArrangement = Arrangement.spacedBy(8.dp),
                    horizontalAlignment = Alignment.CenterHorizontally,
                    modifier = Modifier.fillMaxWidth(),
                    contentPadding = PaddingValues(16.dp),
                ) {
                    items(collectEvents) { event ->
                        AppText(
                            textContent = event,
                            fontSize = 18.sp
                        )
                    }
                }
            } ?: KrillStripedLoader()
        }

        Spacer(Modifier.height(10.dp))

        LazyColumn(
            verticalArrangement = Arrangement.spacedBy(8.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
            modifier = Modifier.fillMaxSize(),
            contentPadding = PaddingValues(16.dp),
        ) {
            item {
                Column(
                    horizontalAlignment = Alignment.CenterHorizontally,
                    verticalArrangement = Arrangement.Top,
                    modifier = Modifier.fillMaxWidth()
                ) {
                    AppText(
                        textContent = "Group Events",
                        fontSize = 25.sp,
                        fontFamily = bungeeHairlineFamily,
                        fontWeight = FontWeight.Black,
                        color = CadmiumOrange
                    )
                    Spacer(Modifier.height(10.dp))

                    if (metadata.activities.isEmpty()) {
                        AppText(
                            textContent = "No group events yet!",
                            fontSize = 16.sp,
                            fontWeight = FontWeight.Black,
                        )
                    }
                }
            }

            items(metadata.activities) { event ->
                AppText(
                    textContent = event.name,
                    fontSize = 18.sp
                )
                Spacer(Modifier.height(4.dp))
                AppText(
                    textContent = event.timestamp,
                    fontSize = 18.sp
                )
            }
        }
    }
}

enum class ComponentState {
    Fetching,
    Done
}

class RustDkgHandler(val sldTld: String, val activityId: String) :
    ActivityListener {

    private val emitter = ActivityEmitter()

    val state = MutableStateFlow<ActivityListenerOutcome?>(null)

    private val scope = CoroutineScope(
        SupervisorJob() + Dispatchers.IO
    )

    init {
        scope.launch {
            emitter.start(
                this@RustDkgHandler,
                sldTld, activityId, getTimezoneOffset()
            )
        }
    }

    fun close() {
        scope.cancel()
    }

    override fun onRecv(value: ActivityListenerOutcome) {
        state.value = value
    }
}