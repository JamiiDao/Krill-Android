package jamiidao.community.krill.notifications_module

import android.app.NotificationChannel
import android.app.NotificationManager
import android.content.Context
import jamiidao.community.krill.RustTypeNotificationImportance
import jamiidao.community.krill.app_log
import jamiidao.community.krill.rustFnNotificationVersioningOps


fun Context.createSigningNotificationChannel(logExisting: Boolean = false) {
    val channelOps = rustFnNotificationVersioningOps()

    val manager =
        getSystemService(NotificationManager::class.java) as NotificationManager

    channelOps.add.forEach { channel ->
        val channel = NotificationChannel(
            channel.channelId,
            channel.channelName,
            parseImportance(channel.importance)
        )


        manager.createNotificationChannel(channel)
    }

    channelOps.remove.forEach { channel ->
        manager.deleteNotificationChannel(channel.channelId)
    }



    if (logExisting) {
        val channels = manager.notificationChannels

        channels.forEach {
            val id = it.id
            val name = it.name

            app_log("id:$id, name:$name")
        }
    }

}

fun parseImportance(importance: RustTypeNotificationImportance): Int {
    return when (importance) {
        RustTypeNotificationImportance.UNSPECIFIED -> NotificationManager.IMPORTANCE_UNSPECIFIED
        RustTypeNotificationImportance.LOW -> NotificationManager.IMPORTANCE_LOW
        RustTypeNotificationImportance.MIN -> NotificationManager.IMPORTANCE_MIN
        RustTypeNotificationImportance.HIGH -> NotificationManager.IMPORTANCE_HIGH
        RustTypeNotificationImportance.NONE -> NotificationManager.IMPORTANCE_NONE
        RustTypeNotificationImportance.DEFAULT -> NotificationManager.IMPORTANCE_DEFAULT
    }
}