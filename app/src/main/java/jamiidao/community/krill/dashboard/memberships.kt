package jamiidao.community.krill.dashboard

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
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
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.HorizontalDivider
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.painter.Painter
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.navigation.NavController
import androidx.navigation.NavHostController
import coil.ImageLoader
import coil.compose.AsyncImage
import coil.decode.SvgDecoder
import jamiidao.community.krill.MainActivity
import jamiidao.community.krill.R
import jamiidao.community.krill.RustFfiException
import jamiidao.community.krill.RustTypeStoredOrgInfoMetadata
import jamiidao.community.krill.ViewGroupActivitiesRoute
import jamiidao.community.krill.components.AppText
import jamiidao.community.krill.components.GlassButton
import jamiidao.community.krill.components.KrillStripedLoader
import jamiidao.community.krill.components.ShowErrorAsBottomSheet
import jamiidao.community.krill.getTimezoneOffset
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
    val timezoneOffset = getTimezoneOffset()

    LaunchedEffect(Unit) {
        try {
            val fetchedMemberships = rustFnGetOrgsMetadata(timezoneOffset)
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
                    modifier = Modifier.fillMaxSize(),
                    verticalArrangement = Arrangement.Top,
                    horizontalAlignment = Alignment.CenterHorizontally
                ) {
                    items(it) { item ->
                        MetadataView(navController, item)
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

@Composable
fun MetadataView(
    navController: NavController,
    metadata: RustTypeStoredOrgInfoMetadata
) {
    val context = LocalContext.current
    val haptic = LocalHapticFeedback.current

    val imageLoader = ImageLoader.Builder(context)
        .components {
            add(SvgDecoder.Factory())
        }
        .build()

    Column(
        verticalArrangement = Arrangement.Center,
        horizontalAlignment = Alignment.Start,
        modifier = Modifier
            .fillMaxWidth(.9f)
            .clickable(
                onClick = {
                    haptic.performHapticFeedback(HapticFeedbackType.ContextClick)
                    navController.navigate(ViewGroupActivitiesRoute(metadata.sldTld))
                }
            )
//            .drawBehind {
//                drawLine(
//                    color = Color.White,
//                    start = Offset(0f, size.height),
//                    end = Offset(size.width, size.height),
//                    strokeWidth = .1.dp.toPx()
//                )
//            }
    ) {
        Row(
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.Center,
            modifier = Modifier.fillMaxWidth()
        ) {
            Box(
                contentAlignment = Alignment.Center,
                modifier = Modifier.weight(0.2f)
            ) {
                AsyncImage(
                    model = metadata.logoIcon,
                    imageLoader = imageLoader,
                    contentDescription = null,
                    modifier = Modifier
                        .padding(10.dp)
                        .size(60.dp)
                        .clip(RoundedCornerShape(50.dp))
                        .border(
                            width = 1.dp,
                            color = Color.Black,
                            shape = RoundedCornerShape(50.dp)
                        )
                )
            }
            Spacer(Modifier.width(10.dp))
            Column(
                verticalArrangement = Arrangement.Center,
                horizontalAlignment = Alignment.Start,
                modifier = Modifier
                    .weight(0.8f)
                    .padding(8.dp)
            ) {
                AppText(textContent = metadata.orgName, fontSize = 30.sp)
                AppText(textContent = metadata.sldTld, fontSize = 15.sp)
            }
        }
    }
}