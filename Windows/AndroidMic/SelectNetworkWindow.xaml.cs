﻿using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Windows;

namespace AndroidMic
{
    /// <summary>
    /// Interaction logic for SelectNetworkWindow.xaml
    /// </summary>
    public partial class SelectNetworkWindow : Window
    {
        public int selectedIdx;

        public SelectNetworkWindow(List<Tuple<string, string>> addresses)
        {
            InitializeComponent();
            selectedIdx = Properties.Settings.Default.SelectNetworkWindow_Idx;
            if (selectedIdx >= addresses.Count)
                selectedIdx = 0;
            // populate combobox list
            NetworkAddressList.Items.Clear();
            foreach (var address in addresses)
                NetworkAddressList.Items.Add($"{address.Item1}: {address.Item2}");
            NetworkAddressList.SelectedIndex = selectedIdx;
        }

        // closing event for SelectNetworkWindow
        private void SelectNetworkWindow_Closing(object sender, CancelEventArgs e)
        {
            DialogResult = true;
        }

        // select in network address list
        private void NetworkAddressList_DropDownClosed(object sender, EventArgs e)
        {
            selectedIdx = NetworkAddressList.SelectedIndex;
            Properties.Settings.Default.SelectNetworkWindow_Idx = selectedIdx;
        }

        // OK button clicked
        private void OKButton_Click(object sender, RoutedEventArgs e)
        {
            Close();
        }
    }
}
