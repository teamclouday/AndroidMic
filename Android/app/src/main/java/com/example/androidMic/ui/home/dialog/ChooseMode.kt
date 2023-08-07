package com.example.androidMic.ui.home.dialog

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.example.androidMic.R
import com.example.androidMic.ui.Dialogs
import com.example.androidMic.ui.MainViewModel
import com.example.androidMic.ui.Modes
import com.example.androidMic.ui.States
import com.example.androidMic.ui.components.ManagerButton

@Composable
fun DialogMode(mainViewModel: MainViewModel, uiStates: States.UiStates) {

    ManagerDialog(
        mainViewModel,
        uiStates,
        Dialogs.Modes
    ) {
        // wifi
        ManagerButton(
            onClick = { mainViewModel.setMode(Modes.WIFI) },
            text = stringResource(id = R.string.mode_wifi),
            modifier = Modifier.fillMaxWidth(0.8f)
        )

        DialogSpacer()

        // bluetooth
        ManagerButton(
            onClick = { mainViewModel.setMode(Modes.BLUETOOTH) },
            text = stringResource(id = R.string.mode_bluetooth),
            modifier = Modifier.fillMaxWidth(0.8f)
        )

        DialogSpacer()

        // usb
        ManagerButton(
            onClick = { mainViewModel.setMode(Modes.USB) },
            text = stringResource(id = R.string.mode_usb),
            modifier = Modifier.fillMaxWidth(0.8f)
        )

        DialogSpacer()

        // udp
        ManagerButton(
            onClick = { mainViewModel.setMode(Modes.UDP) },
            text = stringResource(id = R.string.mode_udp),
            modifier = Modifier.fillMaxWidth(0.8f)
        )
    }
}