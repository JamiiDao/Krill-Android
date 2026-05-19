package jamiidao.community.krill.components

import androidx.compose.foundation.layout.wrapContentWidth
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.sp
import jamiidao.community.krill.ui.theme.White
import jamiidao.community.krill.ui.theme.nunitoFamily


@Composable
fun AppText(
    modifier: Modifier = Modifier,
    textContent: String,
    fontFamily: FontFamily = nunitoFamily,
    fontWeight: FontWeight = FontWeight.Normal,
    fontSize: TextUnit = 30.sp,
    textAlign: TextAlign = TextAlign.Center,
    color: Color = White,
    lineHeight: TextUnit = 35.sp,
    maxLines: Int = Int.MAX_VALUE,
    textOverflow: TextOverflow = TextOverflow.Ellipsis
) {
    Text(
        color = color,
        fontFamily = fontFamily,
        fontWeight = fontWeight,
        textAlign = textAlign,
        text = textContent,
        modifier = modifier.wrapContentWidth(),
        style = TextStyle(
            fontSize = fontSize,
            lineHeight = lineHeight
        ),
        maxLines = maxLines,
        overflow = textOverflow
    )
}