package jamiidao.community.krill.frostServices

import android.app.PendingIntent
import android.app.Service
import android.content.Intent
import android.os.IBinder
import androidx.core.app.NotificationCompat
import jamiidao.community.krill.R
import jamiidao.community.krill.app_log

class FrostDkgHandler : Service() {

    companion object {
        const val ACTION_START = "ACTION_START"
        const val ACTION_STOP = "ACTION_STOP"
        const val CHANNEL_ID = "group-signing"
        const val NOTIF_ID = 1
    }

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        if (intent == null) {
            app_log("Restarted after termination")
            start()
        } else {
            when (intent.action) {
                ACTION_START -> start()
                ACTION_STOP -> stopServiceInternal()
            }
        }

        return START_STICKY
    }

    private fun start() {
        val stopIntent = Intent(this, FrostDkgHandler::class.java).apply {
            action = ACTION_STOP
        }

        val stopPendingIntent = PendingIntent.getService(
            this,
            1001,
            stopIntent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
        )

        val notification = NotificationCompat.Builder(this, CHANNEL_ID)
            .setSmallIcon(R.drawable.signature)
            .setContentTitle("FROST distributed key generation")
            .setContentText("Running background signing protocol")
            .setOngoing(true)
            .addAction(
                R.drawable.signature,
                "STOP",
                stopPendingIntent
            )
            .build()

        startForeground(NOTIF_ID, notification)
    }

    private fun stopServiceInternal() {
        stopForeground(STOP_FOREGROUND_REMOVE)
        stopSelf()
    }
}