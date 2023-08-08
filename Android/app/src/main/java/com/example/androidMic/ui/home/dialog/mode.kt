package com.example.androidMic.ui.home.dialog

import androidx.compose.runtime.Composable
import com.example.androidMic.ui.Dialogs
import com.example.androidMic.ui.MainViewModel
import com.example.androidMic.ui.Modes
import com.example.androidMic.ui.States

@Composable
fun DialogMode(mainViewModel: MainViewModel, uiStates: States.UiStates) {

    ManagerDialog(
        mainViewModel,
        uiStates,
        Dialogs.Modes
    ) {
        DialogList(
            enum = Modes.values().toList(),
            onClick = { mainViewModel.setMode(it) },
            text = { it.toString() }
        )
    }
}