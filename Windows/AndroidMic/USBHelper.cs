using System;
using System.Net;
using System.Net.Sockets;
using System.Windows;
using System.Threading;
using System.Diagnostics;
using System.Collections.Generic;

namespace AndroidMic
{
    enum USBStatus
    {
        DEFAULT,
        LISTENING,
        CONNECTED
    }

    // Reference: https://www.c-sharpcorner.com/article/socket-programming-in-C-Sharp/

    // helper class to connect to device through USB tethering
    class USBHelper
    {
        private readonly int MAX_WAIT_TIME = 1500;
        private readonly int BUFFER_SIZE = 2048;
        private readonly int PORT = 55555;
        private readonly int DEVICE_CHECK_EXPECTED = 123456;
        private readonly int DEVICE_CHECK_DATA = 654321;
        private IPHostEntry mHost;
        private int mSelectedAddressID;
        private Socket mServer;
        public string[] IPAddresses { get; private set; }

        private bool isConnectionAllowed = false;
        private Thread mThreadServer;

        public USBStatus Status { get; private set; } = USBStatus.DEFAULT;

        private readonly MainWindow mMainWindow;
        private readonly AudioData mGlobalData;

        public USBHelper(MainWindow mainWindow, AudioData globalData)
        {
            mMainWindow = mainWindow;
            mGlobalData = globalData;
            RefreshIpAdress();
        }

        // clean before application closes
        public void Clean()
        {
            StopServer();
        }

        // start server and listen for connection
        public void StartServer(int idx)
        {
            StopServer();
            if (idx < 0)
            {
                Application.Current.Dispatcher.Invoke(new Action(() =>
                {
                    mMainWindow.mWaveformDisplay.Reset();
                }));
                AddLog("Server stopped");
                return; // idx < 0 means disabled server
            }
            isConnectionAllowed = true;
            mSelectedAddressID = idx;
            mThreadServer = new Thread(new ThreadStart(Process));
            mThreadServer.Start();
        }

        // accept connection and process data
        private void Process()
        {
            IPAddress ipAddress = IPAddress.Parse(IPAddresses[mSelectedAddressID]);
            IPEndPoint endPoint = new IPEndPoint(ipAddress, PORT);
            // first try to connect to client
            Socket client;
            Status = USBStatus.LISTENING;
            AddLog("Server started (" + IPAddresses[mSelectedAddressID] + ")");
            Debug.WriteLine("[USBHelper] server started");
            try
            {
                mServer = new Socket(ipAddress.AddressFamily, SocketType.Stream, ProtocolType.Tcp);
                mServer.Bind(endPoint);
                mServer.Listen(5); // 5 requests at a time
                do
                {
                    client = mServer.Accept();
                    if (ValidateClient(client)) break;
                    client.Close();
                    client.Dispose();
                } while (isConnectionAllowed);
            } catch(SocketException e)
            {
                Debug.WriteLine("[USBHelper] error: " + e.Message);
                return;
            }
            Status = USBStatus.CONNECTED;
            Debug.WriteLine("[USBHelper] client connected");
            AddLog("Device connected\nclient [Address]: " + client.LocalEndPoint);
            // start processing
            while(isConnectionAllowed && client.Connected)
            {
                try
                {
                    byte[] buffer = new byte[BUFFER_SIZE];
                    int bufferSize = client.Receive(buffer, 0, BUFFER_SIZE, SocketFlags.None);
                    if (bufferSize == 0)
                    {
                        Thread.Sleep(5);
                        break;
                    }
                    else if (bufferSize < 0) break;
                    mGlobalData.AddData(buffer, bufferSize);
                    //Debug.WriteLine("[USBHelper] Process buffer received (" + bufferSize + " bytes)");
                }
                catch (SocketException e)
                {
                    Debug.WriteLine("[USBHelper] Process error: " + e.Message);
                    break;
                }
                Thread.Sleep(1);
            }
            // after that clean socket
            client.Close();
            client.Dispose();
            AddLog("Device disconnected");
            Debug.WriteLine("[USBHelper] client disconnected");
            Status = USBStatus.DEFAULT;
            Application.Current.Dispatcher.Invoke(new Action(() =>
            {
                mMainWindow.mWaveformDisplay.Reset();
            }));
        }

        // stop server
        private void StopServer()
        {
            if (mServer != null)
            {
                mServer.Close();
                mServer.Dispose();
                mServer = null;
            }
            isConnectionAllowed = false;
            if (mThreadServer != null && mThreadServer.IsAlive)
            {
                if (mThreadServer.Join(MAX_WAIT_TIME)) mThreadServer.Abort();
            }
            Debug.WriteLine("[USBHelper] server stopped");
        }

        // check if client is valid
        private bool ValidateClient(Socket client)
        {
            if (client == null || !client.Connected) return false;
            byte[] receivedPack = new byte[4];
            byte[] sentPack = BluetoothHelper.EncodeInt(DEVICE_CHECK_DATA);
            try
            {
                // client.ReceiveTimeout = MAX_WAIT_TIME;
                // client.SendTimeout = MAX_WAIT_TIME;
                // check received integer
                int offset = 0;
                do
                {
                    int sizeReceived = client.Receive(receivedPack, offset, receivedPack.Length-offset, SocketFlags.None);
                    if (sizeReceived <= 0)
                    {
                        Debug.WriteLine("[USBHelper] Invalid client (size received: " + sizeReceived + ")");
                        return false;
                    }
                    offset += sizeReceived;
                } while (offset < 4);
                
                if (BluetoothHelper.DecodeInt(receivedPack) != DEVICE_CHECK_EXPECTED)
                {
                    Debug.WriteLine("[USBHelper] Invalid client (expected: " + DEVICE_CHECK_EXPECTED + ", but get: " + BluetoothHelper.DecodeInt(receivedPack) + ")");
                    return false;
                }
                // send back integer for verification
                client.Send(sentPack, sentPack.Length, SocketFlags.None);
            }
            catch (SocketException e)
            {
                Debug.WriteLine("[USBHelper] ValidateClient error: " + e.Message);
                return false;
            }
            Debug.WriteLine("[USBHelper] Valid client");
            return true;
        }

        // refresh IP adresses
        public bool RefreshIpAdress()
        {
            bool changed = false;
            mHost = Dns.GetHostEntry(Dns.GetHostName());
            List<string> addresses = new List<string>();
            foreach(var ip in mHost.AddressList)
            {
                if (ip.AddressFamily == AddressFamily.InterNetwork)
                    addresses.Add(ip.ToString());
            }
            if(IPAddresses == null || addresses.Count != IPAddresses.Length)
            {
                IPAddresses = new string[addresses.Count];
                changed = true;
            }
            for (int i = 0; i < IPAddresses.Length; i++)
            {
                string address = addresses[i];
                if (IPAddresses[i] != address)
                {
                    changed = true;
                    IPAddresses[i] = address;
                }
            }
            return changed;
        }

        // helper function to add log message
        private void AddLog(string message)
        {
            Application.Current.Dispatcher.Invoke(new Action(() =>
            {
                mMainWindow.AddLogMessage("[USB]\n" + message + "\n");
            }));
        }
    }
}
