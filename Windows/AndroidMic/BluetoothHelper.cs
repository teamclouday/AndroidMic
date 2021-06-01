using System;
using System.IO;
using System.Windows;
using System.Threading;
using System.Diagnostics;
using System.Net.Sockets;
using InTheHand.Net;
using InTheHand.Net.Sockets;
using InTheHand.Net.Bluetooth;

namespace AndroidMic
{
    enum BthStatus
    {
        DEFAULT,
        LISTENING,
        CONNECTED
    }

    class BluetoothHelper
    {
        private readonly Guid mServerUUID = new Guid("34335e34-bccf-11eb-8529-0242ac130003");
        private readonly string mServerName = "Android Microhpone Host";
        private readonly int DEVICE_CHECK_EXPECTED = 123456;
        private readonly int DEVICE_CHECK_DATA = 654321;
        private readonly int MAX_WAIT_TIME = 2000;
        private readonly int BUFFER_SIZE = 2048;

        private BluetoothListener mListener = null;
        private BluetoothClient mClient = null;
        private NetworkStream mClientStream = null;
        private BluetoothEndPoint mTargetDeviceID = null;

        public BthStatus Status { get; private set; } = BthStatus.DEFAULT;

        private void SetStatus(BthStatus value)
        {
            Status = value;
        }

        private bool isConnectionAllowed = false;
        private Thread mProcessThread = null;

        private readonly MainWindow mMainWindow;
        private readonly AudioData mGlobalData;

        public BluetoothHelper(MainWindow mainWindow, AudioData globalData)
        {
            mMainWindow = mainWindow;
            mGlobalData = globalData;
        }

        // start server
        public void StartServer()
        {
            if (mListener == null)
            {
                mListener = new BluetoothListener(mServerUUID)
                {
                    ServiceName = mServerName
                };
                mListener.Start();
            }
            SetStatus(BthStatus.LISTENING);
            Debug.WriteLine("[BluetoothHelper] server started");
            AddLog("Service started listening...");
            Accept();
        }

        // stop server
        public void StopServer()
        {
            isConnectionAllowed = false;
            if (mProcessThread != null && mProcessThread.IsAlive)
            {
                if (!mProcessThread.Join(MAX_WAIT_TIME)) mProcessThread.Abort();
                Disconnect();
            }
            mProcessThread = null;
            Thread.Sleep(500); // wait for accept callback to finish
            if (mListener != null)
            {
                mListener.Server.Dispose();
                mListener.Stop();
                mListener = null;
            }
            SetStatus(BthStatus.DEFAULT);
            AddLog("Service stopped");
            Debug.WriteLine("[BluetoothHelper] server stopped");
        }

        // start accepting clients
        private void Accept()
        {
            isConnectionAllowed = true;
            if (mListener != null)
                mListener.BeginAcceptBluetoothClient(new AsyncCallback(AcceptCallback), mListener);
        }

        // accepting callback
        private void AcceptCallback(IAsyncResult result)
        {
            if(!isConnectionAllowed)
            {
                SetStatus(BthStatus.DEFAULT);
                return;
            }
            if(result.IsCompleted)
            {
                BluetoothClient client = null;
                try
                {
                    client = ((BluetoothListener)result.AsyncState).EndAcceptBluetoothClient(result);
                } catch(Exception e)
                {
                    Debug.WriteLine("[BluetoothHelper] AcceptCallback error: " + e.Message);
                    return;
                }
                Debug.WriteLine("[BluetoothHelper] cliented detected, checking...");
                if (TestClient(client))
                {
                    // close current client
                    client.Dispose();
                    client.Close();
                    Debug.WriteLine("[BluetoothHelper] client valid");
                    // stop alive thread
                    if (mProcessThread != null && mProcessThread.IsAlive)
                    {
                        isConnectionAllowed = false;
                        if (!mProcessThread.Join(MAX_WAIT_TIME)) mProcessThread.Abort();
                        Disconnect();
                    }
                    // dispose stream
                    if(mClientStream != null)
                    {
                        mClientStream.Dispose();
                        mClientStream.Close();
                    }
                    // check for target device ID
                    if(mTargetDeviceID == null)
                    {
                        Accept();
                        return;
                    }
                    // accept client with same ID
                    mClient = mListener.AcceptBluetoothClient();
                    while(!mClient.RemoteEndPoint.Equals(mTargetDeviceID) && isConnectionAllowed)
                    {
                        mClient.Dispose();
                        mClient.Close();
                        mClient = mListener.AcceptBluetoothClient();
                    }
                    // set client stream
                    mClientStream = mClient.GetStream();
                    //if (mClientStream.CanTimeout)
                    //{
                    //    mClientStream.ReadTimeout = MAX_WAIT_TIME;
                    //    mClientStream.WriteTimeout = MAX_WAIT_TIME;
                    //}
                    isConnectionAllowed = true;
                    AddLog("Device connected\nclient [Name]: " + mClient.RemoteMachineName + "\nclient [Address]: " + mClient.RemoteEndPoint);
                    // start processing
                    mProcessThread = new Thread(new ThreadStart(Process));
                    mProcessThread.Start();
                    Debug.WriteLine("[BluetoothHelper] process started");
                }
                else
                {
                    // close current client
                client.Dispose();
                client.Close();
                    Debug.WriteLine("[BluetoothHelper] client invalid");
                    if (client != null) client.Dispose();
                    Accept();
                }
            }
        }

        // check if client is validate
        private bool TestClient(BluetoothClient client)
        {
            if (client == null) return false;
            byte[] receivedPack = new byte[4];
            byte[] sentPack = EncodeInt(DEVICE_CHECK_DATA);
            try
            {
                var stream = client.GetStream();
                if (stream.CanTimeout)
                {
                    stream.ReadTimeout = MAX_WAIT_TIME;
                    stream.WriteTimeout = MAX_WAIT_TIME;
                }
                if (!stream.CanRead || !stream.CanWrite) return false;
                // check received integer
                if (stream.Read(receivedPack, 0, receivedPack.Length) == 0) return false;
                else if (DecodeInt(receivedPack) != DEVICE_CHECK_EXPECTED) return false;
                // send back integer for verification
                stream.Write(sentPack, 0, sentPack.Length);
                // save valid target client ID
                mTargetDeviceID = client.RemoteEndPoint;
                stream.Flush();
                stream.Dispose();
                stream.Close();
            } catch(IOException e)
            {
                Debug.WriteLine("[BluetoothHelper] TestClient error: " + e.Message);
                return false;
            }
            return true;
        }

        // receive audio data
        private void Process()
        {
            SetStatus(BthStatus.CONNECTED);
            while(isConnectionAllowed && IsClientValid())
            {
                try
                {
                    byte[] buffer = new byte[BUFFER_SIZE];
                    int bufferSize = mClientStream.Read(buffer, 0, BUFFER_SIZE);
                    if (bufferSize == 0)
                    {
                        Thread.Sleep(5);
                        break;
                    }
                    mGlobalData.AddData(buffer, bufferSize);
                    //Debug.WriteLine("[BluetoothHelper] Process buffer received (" + bufferSize + " bytes)");
                } catch(IOException e)
                {
                    Debug.WriteLine("[BluetoothHelper] Process error: " + e.Message);
                    break;
                }
                Thread.Sleep(1);
            }
            isConnectionAllowed = false;
            mClientStream.Dispose();
            mClientStream.Close();
            mClientStream = null;
            AddLog("Device disconnected");
            Disconnect();
        }

        // disconnect current client
        private void Disconnect()
        {
            if(mClient != null)
            {
                SetStatus(BthStatus.DEFAULT);
                mClient.Dispose();
                mClient = null;
            }
            Application.Current.Dispatcher.Invoke(new Action(() =>
            {
                mMainWindow.mWaveformDisplay.Reset();
                mMainWindow.ConnectButton.Content = "Connect";
            }));
            Debug.WriteLine("[BluetoothHelper] client disconnected");
        }

        // check if client is valid
        private bool IsClientValid()
        {
            return !(mClient == null || mClient.Connected == false);
        }

        // decode integer from byte array
        public static int DecodeInt(byte[] array)
        {
            if (BitConverter.IsLittleEndian)
                Array.Reverse(array);
            return BitConverter.ToInt32(array, 0);
        }

        // encode integer to byte array
        public static byte[] EncodeInt(int data)
        {
            byte[] array = BitConverter.GetBytes(data);
            if (BitConverter.IsLittleEndian)
                Array.Reverse(array);
            return array;
        }

        // check if bluetooth adapter is enabled
        public static bool CheckBluetooth()
        {
            if(!BluetoothRadio.IsSupported) return false;
            return true;
        }

        // helper function to add log message
        private void AddLog(string message)
        {
            Application.Current.Dispatcher.Invoke(new Action(() =>
            {
                mMainWindow.AddLogMessage("[Bluetooth]\n" + message + "\n");
            }));
        }
    }
}
