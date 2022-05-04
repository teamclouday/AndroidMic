using System;
using System.IO;
using System.Text;
using System.Threading;
using System.Diagnostics;
using InTheHand.Net;
using InTheHand.Net.Sockets;
using InTheHand.Net.Bluetooth;
using AndroidMic.Audio;

namespace AndroidMic.Streaming
{
    public class StreamerBluetooth : Streamer
    {
        private readonly string TAG = "StreamerBluetooth";
        private readonly Guid serverUUID = new Guid("34335e34-bccf-11eb-8529-0242ac130003");
        private readonly string serverName = "AndroidMic Host";
        
        private readonly BluetoothListener listener;
        private BluetoothClient client;
        private BluetoothEndPoint targetDevice;

        private readonly byte[] buffer;
        private int bufferOffset; // offset for short types (multiple of 2)
        private volatile bool isConnectionAllowed;

        public StreamerBluetooth()
        {
            buffer = new byte[BUFFER_SIZE];
            bufferOffset = 0;
            CheckBluetooth();
            listener = new BluetoothListener(serverUUID)
            {
                ServiceName = serverName
            };
            listener.Start();
            Status = ServerStatus.LISTENING;
            DebugLog("Server Started");
            isConnectionAllowed = true;
            listener.BeginAcceptBluetoothClient(new AsyncCallback(AcceptCallback), listener);
        }

        // async callback for server accept
        private void AcceptCallback(IAsyncResult result)
        {
            if(!isConnectionAllowed)
            {
                Status = ServerStatus.DEFAULT;
                return;
            }
            if(result.IsCompleted)
            {
                try
                {
                    client = listener.EndAcceptBluetoothClient(result);
                } catch(Exception e)
                {
                    DebugLog("AcceptCallback: " + e.Message);
                    listener.BeginAcceptBluetoothClient(new AsyncCallback(AcceptCallback), listener);
                    return;
                }
                DebugLog("AcceptCallback: checking client " + client.RemoteMachineName);
                // validate client
                if(TestClient(client))
                {
                    DebugLog("AcceptCallback: valid client");
                    // close client session
                    client.Dispose();
                    client.Close();
                    // check target device
                    if (targetDevice == null)
                    {
                        listener.BeginAcceptBluetoothClient(new AsyncCallback(AcceptCallback), listener);
                        return;
                    }
                    // wait client with same ID
                    while(isConnectionAllowed)
                    {
                        client.Dispose();
                        client.Close();
                        try
                        {
                            client = listener.AcceptBluetoothClient();
                        } catch (InvalidOperationException e) { return; }
                        if (client.RemoteEndPoint.Equals(targetDevice)) break;
                    }
                    Status = ServerStatus.CONNECTED;
                    DebugLog("AcceptCallback: client connected");
                }
                else
                {
                    client.Dispose();
                    client.Close();
                    DebugLog("AcceptCallback: invalid client");
                    listener.BeginAcceptBluetoothClient(new AsyncCallback(AcceptCallback), listener);
                }
            }
        }

        // shutdown server
        public override void Shutdown()
        {
            isConnectionAllowed = false;
            if (client != null)
            {
                client.Dispose();
                client.Close();
                client = null;
            }
            Thread.Sleep(MIN_WAIT_TIME);
            listener.Server.Dispose();
            listener.Stop();
            DebugLog("Shutdown");
            Status = ServerStatus.DEFAULT;
        }

        // process data
        public override void Process(AudioBuffer sharedBuffer)
        {
            if (!IsAlive()) return;
            try
            {
                var stream = client.GetStream();
                int size = stream.Read(buffer, bufferOffset, BUFFER_SIZE - bufferOffset);
                if (size <= 0) return;
                size += bufferOffset;
                // make sure newData is size of multiple of 2
                if (size % 2 == 1)
                {
                    size -= 1;
                    bufferOffset = 1;
                }
                else bufferOffset = 0;
                byte[] newData = new byte[size];
                Buffer.BlockCopy(buffer, 0, newData, 0, size);
                // if one byte remaining, copy it to the front for next read
                if (bufferOffset == 1)
                    buffer[0] = buffer[size];
                // add data
                sharedBuffer.push(newData);
            }
            catch (IOException e)
            {
                DebugLog("Process: " + e.Message);
            }
            catch (ObjectDisposedException e)
            {
                DebugLog("Process: " + e.Message);
            }
        }

        // check if client is valid
        private bool TestClient(BluetoothClient client)
        {
            if (client == null || !client.Connected) return false;
            byte[] receivePack = new byte[20];
            byte[] sendPack = Encoding.UTF8.GetBytes(DEVICE_CHECK);
            try
            {
                using(var stream = client.GetStream())
                {
                    if(stream.CanTimeout)
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
                    targetDevice = client.RemoteEndPoint;
                    stream.Flush();
                }

            } catch(IOException e)
            {
                DebugLog("TestClient: " + e.Message);
                return false;
            }
            return true;
        }

        // check if streamer is alive
        public override bool IsAlive()
        {
            return Status == ServerStatus.CONNECTED && client != null && client.Connected;
        }

        // get client info
        public override string GetClientInfo()
        {
            if (client == null) return "[null]";
            return "[Name]: " + client.RemoteMachineName + "\n[Address]: " + client.RemoteEndPoint;
        }

        // get server info
        public override string GetServerInfo()
        {
            return "[Name]: " + listener.ServiceName + "\n[Address]: " + listener.Server.LocalEndPoint;
        }

        // check bluetooth availability
        private void CheckBluetooth()
        {
            if (!BluetoothRadio.IsSupported)
                throw new ArgumentException("Bluetooth not enabled");
        }

        // debug log
        private void DebugLog(string message)
        {
            Debug.WriteLine(string.Format("[{0}] {1}", TAG, message));
        }
    }
}
