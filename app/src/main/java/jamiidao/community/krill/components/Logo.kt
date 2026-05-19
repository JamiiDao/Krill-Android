package jamiidao.community.krill.components

import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import jamiidao.community.krill.R
import jamiidao.community.krill.ui.theme.CadmiumOrange
import jamiidao.community.krill.ui.theme.nunitoFamily

@Composable
fun KrillLogo() {
    Row(
        horizontalArrangement = Arrangement.Center,
        verticalAlignment = Alignment.CenterVertically,
        modifier = Modifier.fillMaxWidth()
    ) {

        Image(
            painter = painterResource(R.drawable.krill_logo),
            contentDescription = "Krill Logo",
            modifier = Modifier
                .width(80.dp)
                .padding(0.dp)
        )


        AppText(
            textContent = "Krill",
            fontFamily = nunitoFamily,
            fontSize = 60.sp,
            color = CadmiumOrange
        )
    }
    AppText(
        textContent = "Shared Trust. Seamless Security",
        fontFamily = nunitoFamily,
        fontSize = 18.sp,
        color = CadmiumOrange,
        fontWeight = FontWeight.SemiBold
    )
    Spacer(Modifier.height(10.dp))

}
