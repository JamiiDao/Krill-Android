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
        const val DKG_ROUND1 = "DKG_ROUND1"
        const val DKG_ROUND2 = "DKG_ROUND2"
        const val DKG_FINALIZED = "DKG_FINALIZED"
        const val FINALIZING_DKG = "FINALIZING_DKG"

        const val ACTION_STOP = "ACTION_STOP"
        const val CHANNEL_ID = "group-signing"
        const val NOTIF_ID = 1
    }

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        if (intent == null) {
            app_log("Restarted after termination")
            // TODO start()
        } else {
            when (intent.action) {
                DKG_ROUND1 -> start(
                    contentTitle = "FROST Round1 (automated)",
                    contentText = "Participants are agreeing on a group public key"
                )

                DKG_ROUND2 -> start(
                    contentTitle = "FROST Round2 (automated)",
                    contentText = "Participants are agreeing on a group public key"
                )

                FINALIZING_DKG -> start(
                    contentTitle = "Finalizing FROST Key Agreement (automated)",
                    contentText = "Consolidating group public key"
                )

                DKG_FINALIZED -> start(
                    contentTitle = "Finalized FROST Key Agreement (automated)",
                    contentText = "Participants agreed on a group public key successfully"
                )

                ACTION_STOP -> stopServiceInternal()
            }
        }

        return START_STICKY
    }

    private fun start(
        icon: Int = R.drawable.dkg,
        contentTitle: String,
        contentText: String

    ) {
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
            .setSmallIcon(icon)
            .setContentTitle(contentTitle)
            .setContentText(contentText)
            .setOngoing(true)
            .addAction(
                icon,
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