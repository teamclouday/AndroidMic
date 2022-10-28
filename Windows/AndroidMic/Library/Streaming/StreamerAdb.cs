using AdvancedSharpAdbClient;
using System;
using System.IO;
using System.Windows;
using System.Threading;
using System.Diagnostics;
using System.Net.Sockets;

using AndroidMic.Audio;
using System.Threading.Tasks;


// Reference: https://stackoverflow.com/questions/21748790/how-to-send-a-message-from-android-to-windows-using-usb

namespace AndroidMic.Streaming
{

    // helper class to communicate with android device by USB port
    // use abd.exe
    public class StreamerAdb : Streamer
    {
        private readonly string TAG = "StreamerAdb";

        private readonly AdbServer mServer = new AdbServer();
        private readonly AdvancedAdbClient mAdbClient = new AdvancedAdbClient();
        private DeviceData mDevice = null;
        private TcpClient mTcpClient = null;

        private bool isTryConnectAllowed = false;
        private bool isProcessAllowed = false;
        private bool isForwardCreated = false;

        private Thread mThreadTryConnect = null;
        private Thread mThreadProcess = null;


        private readonly string LOCAL_ADDRESS = "localhost";
        private readonly int port_local = 6000;
        private readonly int port_remote = 6000;


        public StreamerAdb()
        {
            mServer.StartServer("./../../ADB/adb.exe", false);
            Status = ServerStatus.LISTENING;
            isTryConnectAllowed = true;
            TryConnect();
        }

        private void TryConnect()
        {
            while (isTryConnectAllowed)
            {
                if (RefreshDevice())
                {
                    if (isForwardCreated)
                    {
                        mAdbClient.RemoveAllForwards(mDevice);
                        isForwardCreated = false;
                    }
                    // if device is detected, try to start forward tcp
                    if (mAdbClient.CreateForward(mDevice, port_local, port_remote) > 0)
                    {
                        isForwardCreated = true;

                        // if forward tcp created, try to connect with TCP
                        if (Connect())
                        {
                            isProcessAllowed = true;
                            isTryConnectAllowed = false;
                            Status = ServerStatus.CONNECTED;
                            break;
                        }
                        // else remove TCP client and loop again
                        else
                        {
                            isTryConnectAllowed = false;
                            break;
                        }
                    }
                    else
                    {
                        Debug.WriteLine("[ADBHelper] failed to CreateForward on port " + port_local);
                        isTryConnectAllowed = false;
                        isProcessAllowed = false;
                        break;
                    }

                }
            }
        }

        // select the first device if connected
        private bool RefreshDevice()
        {
            var devices = mAdbClient.GetDevices();
            if (devices.Count > 0)
            {
                mDevice = devices[0];
                // clear previous forwards
                mAdbClient.RemoveAllForwards(mDevice);
                mAdbClient.RemoveAllReverseForwards(mDevice);
                return true;
            }
            else
            {
                mDevice = null; // else set to null
                return false;
            }
        }

        // connect to tcp server
        private bool Connect()
        {
            Debug.WriteLine("[ADBHelper] Trying to connect");
            mTcpClient = new TcpClient();
            var connection = mTcpClient.BeginConnect(LOCAL_ADDRESS, port_local, null, null);
            // wait for connection for 1 second
            if (!connection.AsyncWaitHandle.WaitOne(TimeSpan.FromSeconds(1)))
            {
                // failed to connect to server, meaning no server available
                return false;
            }
            mTcpClient.EndConnect(connection);
            Debug.WriteLine("[ADBHelper] mTcpClient.Connected = " + mTcpClient.Connected);
            return mTcpClient.Connected;
        }

        


        public override async Task Process(AudioBuffer sharedBuffer)
        {
            if (!IsAlive()) return;
            int count = 0;

            try
            {
                var stream = mTcpClient.GetStream();
                var result = await sharedBuffer.OpenWriteRegion(BUFFER_SIZE);
                count = result.Item1;
                var offset = result.Item2;
                count = await stream.ReadAsync(sharedBuffer.Buffer, offset, count);
            }
            catch (IOException e)
            {
                DebugLog("Process: " + e.Message);
                count = 0;
            }
            catch (ObjectDisposedException e)
            {
                DebugLog("Process: " + e.Message);
                count = 0;
            }
            finally
            {
                sharedBuffer.CloseWriteRegion(count);
            }
        }



        public override void Shutdown()
        {
            DebugLog("Shutdown");
            // stop try connect thread
            isTryConnectAllowed = false;
            if (mThreadTryConnect != null && mThreadTryConnect.IsAlive)
            {
                if (!mThreadTryConnect.Join(MAX_WAIT_TIME)) mThreadTryConnect.Abort();
            }
            // stop process thread
            isProcessAllowed = false;
            if (mThreadProcess != null && mThreadProcess.IsAlive)
            {
                if (!mThreadProcess.Join(MAX_WAIT_TIME)) mThreadProcess.Abort();
            }
            // disconnect
            if (mTcpClient != null)
            {
                mTcpClient.Close();
                mTcpClient.Dispose();
                mTcpClient = null;
            }
            Debug.WriteLine("[ADBHelper] disconnected");
            // stop forwarding
            mAdbClient.RemoveAllForwards(mDevice);
        }


        public override string GetClientInfo()
        {
            return "GetClientInfo";
        }

        public override string GetServerInfo()
        {
            return "GetServerInfo";
        }

        public override bool IsAlive()
        {
            return (Status == ServerStatus.CONNECTED) && (mAdbClient != null) &&
                (mTcpClient != null) && (mTcpClient.Connected == true) &&
                (mTcpClient.Client != null) && (mTcpClient.Client.Connected);
        }

        // debug log
        private void DebugLog(string message)
        {
            Debug.WriteLine(string.Format("[{0}] {1}", TAG, message));
        }
    }
}
