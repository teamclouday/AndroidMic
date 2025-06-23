package io.github.teamclouday.AndroidMic.domain.service


import android.app.Notification
import android.app.PendingIntent
import android.content.Intent
import android.widget.Toast
import androidx.core.app.NotificationCompat
import io.github.teamclouday.AndroidMic.R
import io.github.teamclouday.AndroidMic.ui.MainActivity
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch

const val STOP_STREAM_ACTION = "STOP_STREAM_ACTION"
const val CHANNEL_ID = "Service"

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


        val stopStreamingIntent: Intent = Intent(ctx, ForegroundService::class.java)
            .setAction(STOP_STREAM_ACTION)

        val pStopStreamingIntent = PendingIntent.getService(
            ctx, 0, stopStreamingIntent, PendingIntent.FLAG_IMMUTABLE
        )

        val builder = NotificationCompat.Builder(ctx, CHANNEL_ID)
            .setSmallIcon(R.mipmap.ic_launcher)
            .setContentTitle(ctx.getString(R.string.app_name))
            .setContentText(ctx.getString(R.string.notification_info))
            .setPriority(NotificationCompat.PRIORITY_DEFAULT)
            .setContentIntent(pLaunchIntent)
            .setOngoing(true)
            .addAction(
                NotificationCompat.Action(
                    R.drawable.ic_launcher_foreground,
                    ctx.getString(R.string.stop_streaming),
                    pStopStreamingIntent
                )
            );

        return builder.build()
    }
}

