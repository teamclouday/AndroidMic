package com.example.androidMic.ui.home.dialog

import android.os.Build
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import com.example.androidMic.R
import com.example.androidMic.ui.Dialogs
import com.example.androidMic.ui.MainViewModel
import com.example.androidMic.ui.States
import com.example.androidMic.ui.Themes
import com.example.androidMic.ui.components.ManagerCheckBox

@Composable
fun DialogTheme(mainViewModel: MainViewModel, uiStates: States.UiStates) {

    ManagerDialog(
        mainViewModel,
        uiStates,
        Dialogs.Themes
    ) {
        DialogList(
            enum = Themes.values().toList(),
            onClick = { mainViewModel.setTheme(it) },
            text = { it.toString() }
        )

        if (Build.VERSION.SDK_INT > Build.VERSION_CODES.S) {
            DialogDivider()

            ManagerCheckBox(
                checked = uiStates.dynamicColor,
                onClick = {
                    mainViewModel.setDynamicColor(it)
                },
                text = stringResource(id = R.string.dynamic_color)
            )
        }
    }

}
