package jamiidao.community.krill

import android.app.Application
import android.content.Context
import android.util.Log
import androidx.camera.core.impl.utils.ContextUtil.getApplication
import java.time.ZoneId
import java.time.ZonedDateTime
import kotlin.text.startsWith

fun app_log(message: String) {
    Log.d("KRILL> ", message)
}

fun isValidDeeplink(value: String): Boolean {
    return value.startsWith("krill://join") || value.startsWith("krill://dkg")
}

fun appStoragePath(context: Context): String {
    return context.filesDir.absolutePath
}

fun getTimezoneOffset(): Int {
    return ZonedDateTime.now(ZoneId.systemDefault()).offset.totalSeconds
}