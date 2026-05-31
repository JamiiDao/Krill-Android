package jamiidao.community.krill.components

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.wrapContentWidth
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import jamiidao.community.krill.AppNavigation
import jamiidao.community.krill.ui.theme.CadmiumOrange
import jamiidao.community.krill.ui.theme.ModalColor
import jamiidao.community.krill.ui.theme.MysticBlack
import kotlinx.coroutines.launch

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ShowErrorAsBottomSheet(
    title: String,
    error: String,
    imageID: Int,
    imageDescription: String,
    buttonTextContent: String,
    callback: () -> Unit
) {
    val scope = rememberCoroutineScope()
    val openBottomSheet = rememberSaveable { mutableStateOf(true) }

    val bottomSheetState =
        rememberModalBottomSheetState(skipPartiallyExpanded = true)

    // Sheet content
    if (openBottomSheet.value) {


        ModalBottomSheet(
            onDismissRequest = { openBottomSheet.value = false },
            sheetState = bottomSheetState,
            containerColor = ModalColor
        ) {
            val scrollState = rememberScrollState()

            Column(
                modifier = Modifier
                    .wrapContentWidth()
                    .verticalScroll(scrollState),
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.Center
            ) {
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(20.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.Center
                ) {
                    Image(
                        painter = painterResource(id = imageID),
                        contentDescription = imageDescription,
                        modifier = Modifier
                            .size(50.dp)
                            .padding(5.dp)
                    )

                    AppText(
                        textContent = title,
                        fontSize = 40.sp,
                        color = CadmiumOrange
                    )
                }
                Box(
                    contentAlignment = Alignment.Center,
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(20.dp)
                        .defaultMinSize(minHeight = 100.dp)
                ) {
                    AppText(textContent = error, fontSize = 20.sp)
                }

                Row(
                    Modifier
                        .fillMaxWidth()
                        .padding(20.dp),
                    horizontalArrangement = Arrangement.Center,
                    verticalAlignment = Alignment.Bottom
                ) {
                    GlassButton(
                        callback = {
                            scope
                                .launch { bottomSheetState.hide() }
                                .invokeOnCompletion {
                                    callback()

                                    if (!bottomSheetState.isVisible) {
                                        openBottomSheet.value = false
                                    }
                                }
                        },
                        textContent = buttonTextContent,
                    )
                }
            }
        }
    }
}

