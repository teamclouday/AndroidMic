package com.example.androidMic.ui.home

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Divider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import com.example.androidMic.R
import com.example.androidMic.ui.Dialogs
import com.example.androidMic.ui.MainViewModel
import com.example.androidMic.ui.States
import com.example.androidMic.ui.home.dialog.DialogAudioFormat
import com.example.androidMic.ui.home.dialog.DialogChannelCount
import com.example.androidMic.ui.home.dialog.DialogMode
import com.example.androidMic.ui.home.dialog.DialogSampleRate
import com.example.androidMic.ui.home.dialog.DialogTheme
import com.example.androidMic.ui.home.dialog.DialogWifiIpPort

data class MenuItem(
    val id: Dialogs,
    val title: String,
    val subTitle: String,
    val contentDescription: String,
    val icon: Int?
)

@Composable
fun DrawerBody(mainViewModel: MainViewModel, uiStates: States.UiStates) {

    // maybe this could be optimize, idk
    DialogWifiIpPort(mainViewModel = mainViewModel, uiStates = uiStates)
    DialogMode(mainViewModel = mainViewModel, uiStates = uiStates)
    DialogTheme(mainViewModel = mainViewModel, uiStates = uiStates)
    DialogSampleRate(mainViewModel = mainViewModel, uiStates = uiStates)
    DialogChannelCount(mainViewModel = mainViewModel, uiStates = uiStates)
    DialogAudioFormat(mainViewModel = mainViewModel, uiStates = uiStates)

    val recordItems = listOf(
        MenuItem(
            id = Dialogs.SampleRates,
            title = stringResource(id = R.string.sample_rate),
            subTitle = uiStates.sampleRate.value.toString(),
            contentDescription = "set sample rate",
            icon = null
        ),
        MenuItem(
            id = Dialogs.ChannelCount,
            title = stringResource(id = R.string.channel_count),
            subTitle = uiStates.channelCount.toString(),
            contentDescription = "set channel count",
            icon = null
        ),
        MenuItem(
            id = Dialogs.AudioFormat,
            title = stringResource(id = R.string.audio_format),
            subTitle = uiStates.audioFormat.toString(),
            contentDescription = "set audio format",
            icon = null
        ),
    )

    val connectionItems = listOf(
        MenuItem(
            id = Dialogs.Modes,
            title = stringResource(id = R.string.drawerMode),
            subTitle = uiStates.mode.toString(),
            contentDescription = "set mode",
            icon = R.drawable.settings_24px
        ),
        MenuItem(
            id = Dialogs.IpPort,
            title = stringResource(id = R.string.drawerIpPort),
            subTitle = uiStates.ip + ":" + uiStates.port,
            contentDescription = "set ip and port",
            icon = R.drawable.wifi_24px

        ),
    )

    val otherItems = listOf(
        MenuItem(
            id = Dialogs.Themes,
            title = stringResource(id = R.string.drawerTheme),
            subTitle = uiStates.theme.toString(),
            contentDescription = "set theme",
            icon = R.drawable.dark_mode_24px
        )
    )

    LazyColumn(
        modifier = Modifier
            .fillMaxHeight()
            .width(355.dp)
            .background(color = MaterialTheme.colorScheme.background)
    ) {
        // setting title
        item {
            Box(
                modifier = Modifier
                    .padding(vertical = 64.dp)
                    .padding(start = 25.dp)
            ) {
                Text(
                    text = stringResource(id = R.string.drawerHeader),
                    style = MaterialTheme.typography.titleLarge,
                    color = MaterialTheme.colorScheme.onBackground
                )
            }
        }

        item {
            SettingsItemsSubtitle(R.string.drawer_subtitle_connection)
        }

        items(connectionItems) { item ->
            SettingsItem(mainViewModel, item)
        }

        item {
            SettingsItemsSubtitle(R.string.drawer_subtitle_record)
        }

        items(recordItems) { item ->
            SettingsItem(mainViewModel, item)
        }

        item {
            SettingsItemsSubtitle(R.string.drawer_subtitle_other)
        }

        items(otherItems) { item ->
            SettingsItem(mainViewModel, item)
        }
    }
}


@Composable
private fun SettingsItemsSubtitle(
    subtitle: Int
) {
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .padding(start = 10.dp, top = 25.dp, bottom = 10.dp)
    ) {
        Text(
            text = stringResource(id = subtitle),
            style = MaterialTheme.typography.titleMedium,
            color = MaterialTheme.colorScheme.onBackground
        )
    }
    Divider(color = MaterialTheme.colorScheme.onBackground)
}

@Composable
private fun SettingsItem(mainViewModel: MainViewModel, item: MenuItem) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clickable {
                mainViewModel.showDialog(item.id)
            }
            .padding(16.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        if (item.icon != null) {
            Icon(
                painter = painterResource(id = item.icon),
                contentDescription = item.contentDescription,
                tint = MaterialTheme.colorScheme.onBackground
            )
        }
        Spacer(modifier = Modifier.width(16.dp))
        Column {
            Text(
                text = item.title,
                style = MaterialTheme.typography.bodyLarge,
                color = MaterialTheme.colorScheme.onBackground
            )

            Text(
                text = item.subTitle,
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onBackground
            )
        }
    }
    Divider(color = MaterialTheme.colorScheme.onBackground)
}