package jamiidao.community.krill.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.MutableState
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.platform.LocalHapticFeedback

@Composable
fun GlassButton(
    textContent: String,
    callback: () -> Unit,
    width: Float = 0.8f,
    filled: Boolean = false
) {

    val haptic = LocalHapticFeedback.current

    Box(
        contentAlignment = Alignment.Center,
        modifier = Modifier
            .fillMaxWidth(width)
            .clickable {
                haptic.performHapticFeedback(
                    HapticFeedbackType.LongPress
                )
                callback()
            }

    ) {
        KrillBorderRing(
            borderColor = Color(0xFFFF6600),
            shape = RoundedCornerShape(percent = 50)
        ) {
            KrillGlassSurface(
                content = {
                    Text(
                        color = Color.White,
                        text = textContent,
                    )
                },
                filled = filled
            )
        }
    }

}


@Composable
fun GlassButtonWithComposable(
    content: @Composable () -> Unit,
    callback: () -> Unit,
    width: Float = 0.8f,
    filled: Boolean = false,
    enabled: MutableState<Boolean>
) {

    val haptic = LocalHapticFeedback.current

    Box(
        contentAlignment = Alignment.Center,
        modifier = Modifier
            .fillMaxWidth(width)
            .clickable {
                haptic.performHapticFeedback(
                    HapticFeedbackType.LongPress
                )
                callback()
            }

    ) {
        KrillBorderRing(
            borderColor = Color(0xFFFF6600),
            shape = RoundedCornerShape(percent = 50)
        ) {
            if (enabled.value) {
                KrillGlassSurface(
                    content = {
                        content()
                    },
                    filled = filled
                )
            } else {
                KrillStripedLoader(modifier = Modifier.fillMaxWidth())
            }
        }
    }

}