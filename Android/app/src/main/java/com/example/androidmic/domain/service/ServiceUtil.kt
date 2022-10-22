package com.example.androidmic.domain.service

import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.os.Bundle
import android.os.Message
import android.os.Messenger
import android.widget.Toast
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat
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


class MessageUi(private val ctx: Context) {
    // show message on UI
    fun showMessage(message: String) {
        CoroutineScope(Dispatchers.Main).launch {
            Toast.makeText(ctx, message, Toast.LENGTH_SHORT).show()
        }
    }

    // id : 0 for audioStream, 1 for audioRecord
    fun showNotification(contentText: String, id: Int) {
        CoroutineScope(Dispatchers.Main).launch {
            val intent = Intent(ctx, MainActivity::class.java).apply {
                flags =
                    (Intent.FLAG_ACTIVITY_SINGLE_TOP or Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_BROUGHT_TO_FRONT)
            }
            val pendingIntent =
                PendingIntent.getActivity(ctx, 0, intent, PendingIntent.FLAG_IMMUTABLE)

            val builder = NotificationCompat.Builder(ctx, "service")
                .setSmallIcon(R.drawable.icon)
                .setContentTitle(ctx.getString(R.string.app_name))
                .setContentText(contentText)
                .setPriority(NotificationCompat.PRIORITY_DEFAULT)
                .setContentIntent(pendingIntent)
                .setOngoing(true)

            with(NotificationManagerCompat.from(ctx))
            {
                notify(id, builder.build())
            }
        }
    }

    // id : 0 for audioStream, 1 for audioRecord, 2 for all
    fun removeNotification(id: Int) {
        CoroutineScope(Dispatchers.Main).launch {
            with(NotificationManagerCompat.from(ctx))
            {
                if (id == 2)
                    cancelAll()
                else
                    cancel(id)
            }
        }
    }
}

