package jamiidao.community.krill.ui.theme

import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.TileMode

val CadmiumOrange = Color(0xFFFF6600)
val MysticBlack = Color(0xFF0B0414)
val White = Color(0xFFDFDFDF)

val brushDarkVerticalGradient = Brush.verticalGradient(
    endY = Float.POSITIVE_INFINITY,
    colorStops = arrayOf(
        0.0f to Color(0xFF3A1951),
        0.40f to Color(0xFF1F072A),
        0.6f to Color(0xFF14001A),
        1.0f to Color(0xFF09000C)
    )
)

val brushDarkHorizontalGradient = Brush.horizontalGradient(
    colors = listOf(CadmiumOrange, MysticBlack),
    startX = 0.0f,
    endX = Float.POSITIVE_INFINITY,
    tileMode = TileMode.Clamp
)
