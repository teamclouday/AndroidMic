using System;
using System.IO;
using System.Net;
using System.Linq;
using System.Text;
using System.Net.Sockets;
using System.Threading;
using System.Threading.Tasks;
using System.Diagnostics;
using AdvancedSharpAdbClient;
using AdvancedSharpAdbClient.Exceptions;
using AndroidMic.Audio;

// Reference: https://stackoverflow.com/questions/21748790/how-to-send-a-message-from-android-to-windows-using-usb

namespace AndroidMic.Streaming
{

    // helper class to communicate with android device by USB port
    // use abd.exe
    public class StreamerAdb : Streamer
    {
        private readonly string TAG = "StreamerAdb";

        private readonly AdbServer adbServer;
        private readonly AdvancedAdbClient adbClient;

        private readonly Socket listener;
        private Socket client;

        private bool isRunAdbAllowed = false;
        private bool isConnectionAllowed = false;

        private readonly int androidPort = 6000; // for Android
        private int pcPort = 5999; // for PC

        private readonly int REFRESH_FREQ = 500; // refresh adb info every 0.5s
        private readonly string ADB_EXE_PATH = "ADB/adb.exe"; // this file is always copied to output path

        public StreamerAdb()
        {
            Debug.WriteLine("[StreamerAdb] init");

            // prepare TCP server socket
            listener = new Socket(AddressFamily.InterNetwork, SocketType.Stream, ProtocolType.Tcp);
            BindPort();
            listener.Listen(5);
            Status = ServerStatus.LISTENING;
            isConnectionAllowed = true;
            listener.BeginAccept(AcceptCallback, listener);

            // prepare adb connection
            adbServer = new AdbServer();
            adbClient = new AdvancedAdbClient();

            // start async Task to start adb server
            // and detect connected device
            isRunAdbAllowed = true;
            _ = Task.Factory.StartNew(RunAdb, TaskCreationOptions.LongRunning);
        }

        // loop and select a valid port
        private void BindPort()
        {
            int p = pcPort;
            for (; p <= 65535; p++)
            {
                DebugLog("SelectPort: testing port " + p);
                try
                {
                    listener.Bind(new IPEndPoint(IPAddress.Loopback, p));
                }
                catch (SocketException e)
                {
                    DebugLog("SelectPort: port " + p + " invalid, " + e.Message);
                    continue;
                }
                catch (ObjectDisposedException e)
                {
                    DebugLog("SelectPort: listener has been disposed " + e.Message);
                    break;
                }
                DebugLog("SelectPort: valid port " + p);
                break;
            }
            if (p > 65535)
                throw new ArgumentException("No valid port can be found for localhost");
            pcPort = p;
        }

        // check adb info
        private async void RunAdb()
        {
            while (isRunAdbAllowed)
            {
                // first start adb server
                try
                {
                    adbServer.StartServer(ADB_EXE_PATH, false);
                }
                catch (AdbException e)
                {
                    DebugLog($"Failed to start adb server! {e}");
                    await Task.Delay(REFRESH_FREQ);
                    continue;
                }

                // refresh for all devices (only if not connected)
                if (Status != ServerStatus.CONNECTED)
                {
                    RefreshDevices();
                }

                await Task.Delay(REFRESH_FREQ);
            }
        }

        // set reverse forwarding on connected devices
        private void RefreshDevices(bool shutdown = false)
        {
            var devices = adbClient.GetDevices();
            foreach (var device in devices)
            {
                if (shutdown)
                {
                    adbClient.RemoveAllReverseForwards(device);
                }
                else
                {
                    var remote = $"tcp:{androidPort}";
                    var local = $"tcp:{pcPort}";
                    // create reverse forward on all connected devices
                    var reversed = adbClient.ListReverseForward(device);
                    if (!reversed.Any(p => p.Remote.Equals(local) && p.Local.Equals(remote)))
                    {
                        try
                        {
                            // here adbClient's "remote" parameter should be our "local"
                            adbClient.CreateReverseForward(device, remote, local, false);
                            DebugLog($"Created reverse forward from {remote} to {local} with device ({device.Model})");
                        }
                        catch (Exception e)
                        {
                            DebugLog($"Failed to create reverse forward from {remote} to {local} with device ({device.Model})\n{e}");
                        }
                    }
                }
            }
        }

        // async callback for server accept
        private void AcceptCallback(IAsyncResult result)
        {
            if (!isConnectionAllowed)
            {
                Status = ServerStatus.DEFAULT;
                return;
            }
            if (result.IsCompleted)
            {
                try
                {
                    client = listener.EndAccept(result);
                }
                catch (Exception e)
                {
                    DebugLog("AcceptCallback: " + e.Message);
                    listener.BeginAccept(AcceptCallback, listener);
                    return;
                }
                DebugLog("AcceptCallback: checking client " + client.RemoteEndPoint);
                // validate client
                if (TestClient(client))
                {
                    DebugLog("AcceptCallback: valid client");
                    Status = ServerStatus.CONNECTED;
                    DebugLog("AcceptCallback: client connected");
                }
                else
                {
                    client.Dispose();
                    client.Close();
                    DebugLog("AcceptCallback: invalid client");
                    listener.BeginAccept(AcceptCallback, listener);
                }
            }
        }

        // shutdown server
        public override void Shutdown()
        {
            // shutdown TCP server
            isConnectionAllowed = false;
            if (client != null)
            {
                client.Dispose();
                client.Close();
                client = null;
            }
            Thread.Sleep(MIN_WAIT_TIME);
            listener.Dispose();
            listener.Close();

            // shutdown adb
            isRunAdbAllowed = false;
            RefreshDevices(true);

            DebugLog("Shutdown");
            Status = ServerStatus.DEFAULT;
        }

        // process data
        public override async Task Process(AudioBuffer sharedBuffer)
        {
            if (!IsAlive()) return;
            int count = 0;
            try
            {
                var stream = new NetworkStream(client);
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

        // check if client is valid
        private bool TestClient(Socket client)
        {
            if (client == null || !client.Connected) return false;
            byte[] receivePack = new byte[20];
            byte[] sendPack = Encoding.UTF8.GetBytes(DEVICE_CHECK);
            try
            {
                using (var stream = new NetworkStream(client))
                {
                    if (stream.CanTimeout)
                    {
                        stream.ReadTimeout = MAX_WAIT_TIME;
                        stream.WriteTimeout = MAX_WAIT_TIME;
                    }
                    if (!stream.CanRead || !stream.CanWrite) return false;
                    // check received bytes
                    int size = stream.Read(receivePack, 0, receivePack.Length);
                    if (size <= 0) return false;
                    if (!Encoding.UTF8.GetString(receivePack, 0, size).Equals(DEVICE_CHECK_EXPECTED)) return false;
                    // send back bytes
                    stream.Write(sendPack, 0, sendPack.Length);
                    stream.Flush();
                }
            }
            catch (IOException e)
            {
                DebugLog("TestClient: " + e.Message);
                return false;
            }
            return true;
        }

        // get client info
        public override string GetClientInfo()
        {
            if (client == null) return "[null]";
            return "[Adb Client]: " + client.RemoteEndPoint;
        }

        // get server info
        public override string GetServerInfo()
        {
            return $"[Android Port]: {androidPort}";
        }

        // check if streamer is alive
        public override bool IsAlive()
        {
            return Status == ServerStatus.CONNECTED && client != null && client.Connected;
        }

        // debug log
        private void DebugLog(string message)
        {
            Debug.WriteLine(string.Format("[{0}] {1}", TAG, message));
        }
    }
}
