package jamiidao.community.krill.notifications_module

import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Intent
import android.graphics.BitmapFactory
import android.net.Uri
import android.util.Log
import androidx.compose.runtime.rememberCoroutineScope
import androidx.core.app.NotificationCompat
import com.google.firebase.messaging.FirebaseMessagingService
import com.google.firebase.messaging.RemoteMessage
import jamiidao.community.krill.MainActivity
import jamiidao.community.krill.R
import jamiidao.community.krill.RustTypeFetchedNotificationInfo
import jamiidao.community.krill.RustTypeNotificationIconType
import jamiidao.community.krill.RustTypeNotificationImportance
import jamiidao.community.krill.RustTypeReceivedNotificationData
import jamiidao.community.krill.app_log
import jamiidao.community.krill.rustFnProcessNotificationInfo
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import androidx.core.net.toUri

class FGCMService : FirebaseMessagingService() {

    private val scope = CoroutineScope(Dispatchers.IO + SupervisorJob())


    override fun onMessageReceived(message: RemoteMessage) {
        scope.launch {
            val info = RustTypeReceivedNotificationData(
                data = message.data["base64_encoded"],
                messageId = message.messageId,
                from = message.from,
                originalPriority = message.originalPriority,
                priority = message.priority,
                senderId = message.senderId,
                sentTime = message.sentTime,
                ttl = message.ttl
            )

            val processNotification = rustFnProcessNotificationInfo(info)

            processNotification?.let {
                sendNotification(it)
            }
        }
    }

    override fun onNewToken(token: String) {
        app_log(token)
        // send token to backend here
    }

    private fun sendNotification(message: RustTypeFetchedNotificationInfo) {
        app_log("Processed: $message")

        val intent = Intent(
            Intent.ACTION_VIEW,
            "krill://${message.groupEventId}".toUri()
        ).apply {
            addFlags(
                Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TOP
            )
        }

        val pendingIntent = PendingIntent.getActivity(
            this,
            0,
            intent,
            PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT
        )

        val bitmap = BitmapFactory.decodeResource(
            resources,
            R.drawable.krill_white
        )

        val builder = NotificationCompat.Builder(this, message.channelInfo.channelId)
            .setSmallIcon(fetchNotificationIcon(message.channelInfo.iconType))
            .setLargeIcon(bitmap)
            .setStyle(
                NotificationCompat.BigPictureStyle()
                    .bigLargeIcon(bitmap)
                    .bigPicture(bitmap)
            )
            .setContentTitle(message.heading)
            .setContentText(message.subheading)
            .setAutoCancel(true)
            .setContentIntent(pendingIntent)

        val manager = getSystemService(NotificationManager::class.java)

        manager.notify(message.notificationId, builder.build())
    }
}

fun fetchNotificationIcon(iconType: RustTypeNotificationIconType): Int {
    return when (iconType) {
        RustTypeNotificationIconType.DEFAULT -> R.drawable.krill_white
        RustTypeNotificationIconType.SIGNING -> R.drawable.signature
    }
}