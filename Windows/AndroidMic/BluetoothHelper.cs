using System;
using System.IO;
using System.Windows;
using System.Threading;
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
        private readonly int MAX_WAIT_TIME = 1500;

        private BluetoothListener mListener = null;
        private BluetoothClient mClient = null;

        public BthStatus Status { get; private set; } = BthStatus.DEFAULT;
        private bool isConnectionAllowed = false;
        private Thread mProcessThread = null;
        private readonly MainWindow mMainWindow;

        public BluetoothHelper(MainWindow mainWindow) { mMainWindow = mainWindow; }

        public void StartServer()
        {
            if (mListener != null) mListener.Stop();
            mListener = new BluetoothListener(mServerUUID);
            mListener.ServiceName = mServerName;
            mListener.Start();
            Status = BthStatus.LISTENING;
            AddLog("Service started listening...");
            Accept();
        }

        public void StopServer()
        {
            isConnectionAllowed = false;
            if (mProcessThread != null && mProcessThread.IsAlive)
            {
                if (!mProcessThread.Join(MAX_WAIT_TIME)) mProcessThread.Abort();
                Disconnect();
            }
            mProcessThread = null;
            if (mListener != null)
            {
                mListener.Server.Dispose();
                mListener.Stop();
                mListener = null;
            }
            Status = BthStatus.DEFAULT;
            AddLog("Service stopped");
        }

        private void Accept()
        {
            isConnectionAllowed = true;
            if (mListener != null)
                mListener.BeginAcceptBluetoothClient(new AsyncCallback(AcceptCallback), mListener);
        }

        private void AcceptCallback(IAsyncResult result)
        {
            if(!isConnectionAllowed)
            {
                Status = BthStatus.DEFAULT;
                return;
            }
            if(result.IsCompleted)
            {
                BluetoothClient client = ((BluetoothListener)result.AsyncState).EndAcceptBluetoothClient(result);
                if(TestClient(client))
                {
                    if (mProcessThread != null && mProcessThread.IsAlive)
                    {
                        isConnectionAllowed = false;
                        if (!mProcessThread.Join(MAX_WAIT_TIME)) mProcessThread.Abort();
                        Disconnect();
                    }
                    mClient = client;
                    AddLog("Device connected\nclient [Name]: " + mClient.RemoteMachineName + "\n client [Address]: " + mClient.RemoteEndPoint);
                    mProcessThread = new Thread(new ThreadStart(Process));
                    mProcessThread.Start();
                    Process();
                }
                else
                {
                    if (client != null) client.Dispose();
                    Accept();
                }
            }
        }

        private bool TestClient(BluetoothClient client)
        {
            if (client == null) return false;
            byte[] receivedPack = new byte[4];
            byte[] sentPack = EncodeInt(DEVICE_CHECK_DATA);
            try
            {
                var stream = client.GetStream();
                if(stream.CanTimeout)
                {
                    stream.ReadTimeout = MAX_WAIT_TIME;
                    stream.WriteTimeout = MAX_WAIT_TIME;
                }
                if (!stream.CanRead || !stream.CanWrite) return false;
                if (stream.Read(receivedPack, 0, receivedPack.Length) == 0) return false;
                else if (DecodeInt(receivedPack) != DEVICE_CHECK_EXPECTED)
                {
                    int tmp = DecodeInt(receivedPack);
                    return false;
                }
                stream.Write(sentPack, 0, sentPack.Length);
            } catch(IOException)
            {
                return false;
            }
            return true;
        }

        public void Process()
        {
            Status = BthStatus.CONNECTED;
            var stream = mClient.GetStream();
            while(isConnectionAllowed && IsClientValid())
            {
                try
                {

                } catch(IOException)
                {
                    break;
                }
                Thread.Sleep(1);
            }
            Disconnect();
        }

        private void Disconnect()
        {
            if(mClient != null)
            {
                Status = BthStatus.DEFAULT;
                mClient.Dispose();
                mClient = null;
            }
            Accept(); // auto start accepting next client
        }

        private bool IsClientValid()
        {
            return !(mClient == null || mClient.Connected == false);
        }

        public static int DecodeInt(byte[] array)
        {
            if (BitConverter.IsLittleEndian)
                Array.Reverse(array);
            return BitConverter.ToInt32(array, 0);
        }

        public static byte[] EncodeInt(int data)
        {
            byte[] array = BitConverter.GetBytes(data);
            if (BitConverter.IsLittleEndian)
                Array.Reverse(array);
            return array;
        }

        public static float DecodeFloat(byte[] array)
        {
            if (BitConverter.IsLittleEndian)
                Array.Reverse(array);
            return BitConverter.ToSingle(array, 0);
        }

        public static bool CheckBluetooth()
        {
            if(!BluetoothRadio.IsSupported) return false;
            return true;
        }

        private void AddLog(string message)
        {
            Application.Current.Dispatcher.Invoke(new Action(() =>
            {
                mMainWindow.AddLogMessage("[Bluetooth]\n" + message + "\n");
            }));
        }
    }
}
