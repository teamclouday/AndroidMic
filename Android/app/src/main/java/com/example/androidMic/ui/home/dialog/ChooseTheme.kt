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
        // system
        ManagerCheckBox(
            checked = uiStates.theme == Themes.SYSTEM,
            onClick = {
                mainViewModel.setTheme(Themes.SYSTEM)
            },
            text = stringResource(id = R.string.system_theme)
        )

        DialogSpacer()

        // light
        ManagerCheckBox(
            checked = uiStates.theme == Themes.LIGHT,
            onClick = {
                mainViewModel.setTheme(Themes.LIGHT)
            },
            text = stringResource(id = R.string.light_theme)
        )

        DialogSpacer()

        // dark
        ManagerCheckBox(
            checked = uiStates.theme == Themes.DARK,
            onClick = {
                mainViewModel.setTheme(Themes.DARK)
            },
            text = stringResource(id = R.string.dark_theme)
        )

        if (Build.VERSION.SDK_INT > Build.VERSION_CODES.S) {
            DialogDivider()

            // dynamic color
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
