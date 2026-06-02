package jamiidao.community.krill.components

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import jamiidao.community.krill.ui.theme.CadmiumOrange

@Composable
fun KrillGlassSurface(
    modifier: Modifier =
        Modifier,
    show: Boolean = true,
    percentage: Int = 50,
    horizontalPadding: Dp = 20.dp,
    verticalPadding: Dp = 12.dp,
    filled: Boolean = false,
    content: @Composable () -> Unit,
) {
    val shape = RoundedCornerShape(percent = percentage)
    val selectModifier = if (show) {
        modifier
            .fillMaxWidth()
            .clip(shape)
            .shadow(
                elevation = 16.dp,
                shape = shape,
                ambientColor = Color.Black.copy(alpha = 0.85f),
                spotColor = Color.Black.copy(alpha = 0.85f)
            )
            .border(
                width = 1.dp,
                color = Color.White.copy(alpha = 0.12f),
                shape = shape
            )
    } else {
        Modifier
            .fillMaxWidth()
    }

    val selectColor = if (show) {
        Color.Black.copy(alpha = 0.5f)
    } else {
        Color.Transparent
    }

    val selectBackground = if (show) {
        if (filled) {
            Brush.linearGradient(
                colors = listOf(
                    CadmiumOrange.copy(alpha = 0.1f),
                    CadmiumOrange.copy(alpha = 0.08f),
                    CadmiumOrange.copy(alpha = 0.05f)
                )
            )
        } else {
            Brush.linearGradient(
                colors = listOf(
                    Color.White.copy(alpha = 0.1f),
                    Color.White.copy(alpha = 0.08f),
                    Color.White.copy(alpha = 0.05f)
                )
            )
        }
    } else {
        Brush.linearGradient(
            colors = listOf(
                Color.Transparent,
                Color.Transparent
            )
        )
    }

    Surface(
        modifier = selectModifier,
        shape = shape,
        color = selectColor
    ) {
        Box(
            contentAlignment = Alignment.Center,
            modifier = Modifier
                .fillMaxWidth(0.9f)
                .background(
                    selectBackground
                )
                .padding(horizontal = horizontalPadding, vertical = verticalPadding)
        ) {
            content()
        }
    }
}

@Composable
fun KrillBorderRing(
    modifier: Modifier = Modifier,
    shape: Shape = RoundedCornerShape(percent = 50),
    borderColor: Color,
    borderWidth: Dp = .5.dp,
    content: @Composable BoxScope.() -> Unit
) {
    Box(
        modifier = modifier
            .fillMaxWidth(0.8f)
            .border(borderWidth, borderColor, shape)
            .clip(shape)
    ) {
        content()
    }
}


@Composable
fun KrillDotsBackground(
    modifier: Modifier = Modifier,
    dotColor: Color = Color.White,
    dotRadius: Float = 2f,
    spacing: Float = 80f,
) {
    Canvas(
        modifier = modifier
            .background(Color.Black)
    ) {
        val width = size.width
        val height = size.height

        val fadedDotColor = dotColor.copy(alpha = 0.2f)

        var y = 0f
        while (y <= height) {
            var x = 0f
            while (x <= width) {
                drawCircle(
                    color = fadedDotColor,
                    radius = dotRadius,
                    center = Offset(x, y)
                )
                x += spacing
            }
            y += spacing
        }
    }
}
