package jamiidao.community.krill.components

import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.dp
import jamiidao.community.krill.ui.theme.CadmiumOrange


@Composable
fun KrillStripedLoader(
    modifier: Modifier = Modifier,
    color: Color = Color(0xFF421A03),
    fadedColor: Color = CadmiumOrange,
) {
    val transition = rememberInfiniteTransition(label = "")

    val density = LocalDensity.current
    val stripeWidthPx = with(density) { 15.dp.toPx() }

    val offset by transition.animateFloat(
        initialValue = 0f,
        targetValue = stripeWidthPx * 1.5f,
        animationSpec = infiniteRepeatable(
            animation = tween(
                durationMillis = 500,
                easing = LinearEasing
            )
        ),
        label = ""
    )

    Canvas(
        modifier = modifier
            .fillMaxWidth(.8f)
            .height(8.dp)
            .clip(RoundedCornerShape(999.dp))
    ) {
        drawRoundRect(
            color = color,
            cornerRadius = CornerRadius(size.height)
        )

        var startX = -stripeWidthPx * 2 + offset

        while (startX < size.width + stripeWidthPx) {

            drawPath(
                path = Path().apply {
                    moveTo(startX, 0f)
                    lineTo(startX + stripeWidthPx, 0f)
                    lineTo(startX + stripeWidthPx / 2, size.height)
                    lineTo(startX - stripeWidthPx / 2, size.height)
                    close()
                },
                color = fadedColor
            )

            startX += stripeWidthPx * 1.5f
        }
    }
}
