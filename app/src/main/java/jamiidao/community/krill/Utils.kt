package jamiidao.community.krill

import android.util.Log
import kotlin.text.startsWith

fun app_log(message: String) {
    Log.d("KRILL> ", message)
}

fun isValidDeeplink(value: String): Boolean {
    return value.startsWith("krill://join") || value.startsWith("krill://dkg")
}