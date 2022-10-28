package com.example.androidMic.ui.home

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
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
import com.example.androidMic.ui.Event
import com.example.androidMic.ui.MainViewModel
import com.example.androidMic.ui.components.ManagerSetting
import com.example.androidMic.ui.home.dialog.DialogWifiIpPort
import com.example.androidMic.ui.home.dialog.DialogMode
import com.example.androidMic.ui.home.dialog.DialogTheme
import com.example.androidMic.ui.home.dialog.DialogUsbPort
import com.example.androidMic.utils.Modes.Companion.MODE_BLUETOOTH
import com.example.androidMic.utils.Modes.Companion.MODE_USB
import com.example.androidMic.utils.Modes.Companion.MODE_WIFI
import com.example.androidMic.utils.States
import com.example.androidMic.utils.Themes.Companion.DARK_THEME
import com.example.androidMic.utils.Themes.Companion.LIGHT_THEME
import com.example.androidMic.utils.Themes.Companion.SYSTEM_THEME

data class MenuItem(
    val id: Int,
    val title: String,
    val subTitle: String,
    val contentDescription: String,
    val icon: Int
)

@Composable
fun DrawerBody(mainViewModel: MainViewModel, uiStates: States.UiStates) {

    DialogWifiIpPort(mainViewModel = mainViewModel, uiStates = uiStates)
    DialogUsbPort(mainViewModel = mainViewModel, uiStates = uiStates)
    DialogMode(mainViewModel = mainViewModel, uiStates = uiStates)
    DialogTheme(mainViewModel = mainViewModel, uiStates = uiStates)


    val connectionItems = listOf(
        MenuItem(
            id = R.string.drawerMode,
            title = stringResource(id = R.string.drawerMode),
            subTitle = when (uiStates.mode) {
                MODE_WIFI -> stringResource(id = R.string.mode_wifi)
                MODE_BLUETOOTH -> stringResource(id = R.string.mode_bluetooth)
                MODE_USB -> stringResource(id = R.string.mode_usb)
                else -> "NONE"
            },
            contentDescription = "set mode",
            icon = R.drawable.settings_24px
        ),
        MenuItem(
            id = R.string.drawerWifiIpPort,
            title = stringResource(id = R.string.drawerWifiIpPort),
            subTitle = uiStates.IP + ":" + uiStates.PORT,
            contentDescription = "set ip and port",
            icon = R.drawable.wifi_24px

        ),
        MenuItem(
            id = R.string.drawerUsbPort,
            title = stringResource(id = R.string.drawerUsbPort),
            subTitle = uiStates.usbPort,
            contentDescription = "set usb port",
            icon = R.drawable.usb_24px

        )
    )

    val otherItems = listOf(
        MenuItem(
        id = R.string.drawerTheme,
        title = stringResource(id = R.string.drawerTheme),
        subTitle = when (uiStates.theme) {
            SYSTEM_THEME -> stringResource(id = R.string.system_theme)
            LIGHT_THEME -> stringResource(id = R.string.light_theme)
            DARK_THEME -> stringResource(id = R.string.dark_theme)
            else -> "NONE"
        },
        contentDescription = "set theme",
        icon = R.drawable.dark_mode_24px
        )
    )

    LazyColumn {
        // setting title
        item {
            Box(
                modifier = Modifier
                    .fillMaxWidth()
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

        // connection subtitle
        item {
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(10.dp)
            ) {
                Text(
                    text = stringResource(id = R.string.drawer_subtitle_connection),
                    style = MaterialTheme.typography.titleMedium,
                    color = MaterialTheme.colorScheme.onBackground
                )
            }
            Divider(color = MaterialTheme.colorScheme.onBackground)
        }

        items(connectionItems) { item ->
            var shouldShowItem = true
            when (uiStates.mode) {
                MODE_WIFI -> if (item.id == R.string.drawerUsbPort) shouldShowItem = false
                MODE_USB -> if (item.id == R.string.drawerWifiIpPort) shouldShowItem = false
                MODE_BLUETOOTH -> {
                    if (item.id == R.string.drawerUsbPort) shouldShowItem = false
                    if (item.id == R.string.drawerWifiIpPort) shouldShowItem = false
                }
            }
            if (shouldShowItem) {
                ManagerSetting(mainViewModel, item)
                Divider(color = MaterialTheme.colorScheme.onBackground)
            }
        }

        item {
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(start = 10.dp, top = 25.dp, bottom = 10.dp)
            ) {
                Text(
                    text = stringResource(id = R.string.drawer_subtitle_other),
                    style = MaterialTheme.typography.titleMedium,
                    color = MaterialTheme.colorScheme.onBackground
                )
            }
            Divider(color = MaterialTheme.colorScheme.onBackground)
        }
        items(otherItems) { item ->
            ManagerSetting(mainViewModel, item)
            Divider(color = MaterialTheme.colorScheme.onBackground)
        }
    }
}