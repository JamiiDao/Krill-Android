package jamiidao.community.krill.dashboard

import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.painter.Painter
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.navigation.NavController
import androidx.navigation.NavHostController
import jamiidao.community.krill.MainActivity
import jamiidao.community.krill.R
import jamiidao.community.krill.RustFfiException
import jamiidao.community.krill.RustTypeStoredOrgInfoMetadata
import jamiidao.community.krill.components.AppText
import jamiidao.community.krill.components.GlassButton
import jamiidao.community.krill.components.KrillStripedLoader
import jamiidao.community.krill.components.ShowErrorAsBottomSheet
import jamiidao.community.krill.rustFnGetOrgsMetadata
import jamiidao.community.krill.ui.theme.bungeeHairlineFamily


@Composable
fun Memberships(
    mainActivity: MainActivity,
    navController: NavController,
    bottomNavController: NavHostController
) {

    val memberships = remember { mutableStateOf<List<RustTypeStoredOrgInfoMetadata>?>(null) }
    val errorExists = remember { mutableStateOf<String?>(null) }

    LaunchedEffect(Unit) {
        try {
            val fetchedMemberships = rustFnGetOrgsMetadata()
            memberships.value = fetchedMemberships
        } catch (e: RustFfiException) {
            errorExists.value = e.uiMessage();
        }
    }


    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center,
        modifier = Modifier
            .fillMaxSize()
            .padding(8.dp)
    ) {
        memberships.value?.let {
            if (it.isEmpty()) {
                Column(
                    horizontalAlignment = Alignment.CenterHorizontally,
                    verticalArrangement = Arrangement.SpaceAround,
                    modifier = Modifier
                        .fillMaxSize()
                ) {
                    Image(
                        painter = painterResource(R.drawable.baby_birds_singing_in_a_nest),
                        contentDescription = null,
                        modifier = Modifier.size(300.dp)
                    )
                    Spacer(Modifier.height(20.dp))
                    Column(
                        horizontalAlignment = Alignment.CenterHorizontally,
                        verticalArrangement = Arrangement.Center,
                        modifier = Modifier.fillMaxWidth(.8f)
                    ) {
                        AppText(
                            textContent = "No Groups were found just some birds chirps! Try joining one!",
                            fontSize = 20.sp
                        )
                    }

                    GlassButton(
                        textContent = "JOIN GROUP",
                        callback = {
                            bottomNavController.navigate(ROUTE_SCAN_QR)
                        },
                        width = 1f,
                        filled = true,
                    )
                }
            } else {
                LazyColumn(
                    modifier = Modifier.fillMaxSize()
                ) {
                    items(it) { item ->
                        AppText(textContent = item.sldTld)
                    }
                }
            }
        } ?: Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.Center,
            modifier = Modifier.fillMaxSize()
        ) {
            AppText(
                textContent = "Loading groups",
                fontFamily = bungeeHairlineFamily,
                fontSize = 25.sp
            )
            Spacer(Modifier.height(10.dp))
            KrillStripedLoader()
        }
    }

    errorExists.value?.let {
        ShowErrorAsBottomSheet(
            title = "Unable to list Groups",
            error = it,
            imageID = R.drawable.error,
            imageDescription = "",
            buttonTextContent = "Exit App",
            callback = {
                mainActivity.finish()
            },
        )
    }
}