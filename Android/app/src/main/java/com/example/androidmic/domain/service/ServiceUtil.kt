package com.example.androidmic.domain.service


import android.app.Notification
import android.app.PendingIntent
import android.content.Intent
import android.os.Bundle
import android.os.Message
import android.os.Messenger
import android.widget.Toast
import androidx.core.app.NotificationCompat
import com.example.androidmic.AndroidMicApp
import com.example.androidmic.R
import com.example.androidmic.ui.MainActivity
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch


fun reply(sender: Messenger, data: Bundle, what: Int, success: Boolean) {
    data.putBoolean("result", success)
    val reply = Message()
    reply.data = data
    reply.what = what
    sender.send(reply)
}


const val CHANNEL_ID = "Service"
const val ACTION_STOP_SERVICE = "ACTION_STOP_SERVICE"


class MessageUi(private val ctx: ForegroundService) {
    // show message on UI
    fun showMessage(message: String) {
        CoroutineScope(Dispatchers.Main).launch {
            Toast.makeText(ctx, message, Toast.LENGTH_SHORT).show()
        }
    }


    fun getNotification(): Notification {
        // launch activity
        val launchIntent = Intent(ctx, MainActivity::class.java).apply {
            flags =
                (Intent.FLAG_ACTIVITY_SINGLE_TOP or Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_BROUGHT_TO_FRONT)
        }
        val pLaunchIntent =
            PendingIntent.getActivity(ctx, 0, launchIntent, PendingIntent.FLAG_IMMUTABLE)

        // cancel foreground service
        val stopIntent = Intent(ctx, AndroidMicApp.UnboundReceiver::class.java).apply {
            action = ACTION_STOP_SERVICE
        }
        val pStopIntent =
            PendingIntent.getBroadcast(
                ctx,
                0,
                stopIntent,
                PendingIntent.FLAG_CANCEL_CURRENT or PendingIntent.FLAG_IMMUTABLE
            )


        val builder = NotificationCompat.Builder(ctx, CHANNEL_ID)
            .setSmallIcon(R.drawable.icon)
            .setContentTitle(ctx.getString(R.string.app_name))
            .setContentText(ctx.getString(R.string.notification_info))
            .setPriority(NotificationCompat.PRIORITY_DEFAULT)
            .setContentIntent(pLaunchIntent)
            .setOngoing(true)
            .addAction(
                R.drawable.ic_baseline_wifi_24,
                ctx.getString(R.string.notification_stop_service),
                pStopIntent
            )

        return builder.build()
    }
}

