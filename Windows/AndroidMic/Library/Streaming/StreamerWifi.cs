using System;
using System.IO;
using System.Text;
using System.Threading;
using System.Diagnostics;
using System.Net;
using System.Net.Sockets;
using System.Net.NetworkInformation;
using AndroidMic.Audio;

// reference: https://docs.microsoft.com/en-us/dotnet/framework/network-programming/asynchronous-server-socket-example

namespace AndroidMic.Streaming
{
    public class StreamerWifi : Streamer
    {
        private readonly string TAG = "StreamerWifi";

        private string adapterName;
        private readonly Socket listener;
        private Socket client;
        private string address;
        private int port = 55555;

        private readonly byte[] buffer;
        private int bufferOffset;
        private bool isConnectionAllowed = false;

        public StreamerWifi()
        {
            buffer = new byte[BUFFER_SIZE];
            bufferOffset = 0;
            CheckWifi();
            listener = new Socket(AddressFamily.InterNetwork, SocketType.Stream, ProtocolType.Tcp);
            BindPort();
            listener.Listen(5);
            Status = ServerStatus.LISTENING;
            DebugLog("Server Started");
            isConnectionAllowed = true;
            listener.BeginAccept(new AsyncCallback(AcceptCallback), listener);
        }

        // loop and select a valid port
        private void BindPort()
        {
            int p = port;
            for(; p <= 65535; p++)
            {
                DebugLog("SelectPort: testing port " + p);
                try
                {
                    listener.Bind(new IPEndPoint(IPAddress.Parse(address), p));
                } catch(SocketException e)
                {
                    DebugLog("SelectPort: port " + p + " invalid, " + e.Message);
                    continue;
                } catch(ObjectDisposedException e)
                {
                    DebugLog("SelectPort: listener has been disposed " + e.Message);
                    break;
                }
                DebugLog("SelectPort: valid port " + p);
                break;
            }
            if (p > 65535)
                throw new ArgumentException("No valid port can be found for " + address);
            port = p;
        }

        // async callback for server accept
        private void AcceptCallback(IAsyncResult result)
        {
            if(!isConnectionAllowed)
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
                    listener.BeginAccept(new AsyncCallback(AcceptCallback), listener);
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
                    listener.BeginAccept(new AsyncCallback(AcceptCallback), listener);
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
            listener.Dispose();
            listener.Close();
            DebugLog("Shutdown");
            Status = ServerStatus.DEFAULT;
        }

        // process data
        public override void Process(AudioBuffer sharedBuffer)
        {
            if (!IsAlive()) return;
            try
            {
                var stream = new NetworkStream(client);
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
            catch(ObjectDisposedException e)
            {
                DebugLog("Process: " + e.Message);
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
            if (client == null) return "client [null]";
            return "client [Address]: " + client.RemoteEndPoint;
        }

        // get server info
        public override string GetServerInfo()
        {
            return "server [Adapter Name]: " + adapterName + "\nserver [Address]: " + address + "\nserver [Port]: " + port;
        }

        // check if streamer is alive
        public override bool IsAlive()
        {
            return Status == ServerStatus.CONNECTED && client != null && client.Connected;
        }

        // check connectivity, get first main IP
        private void CheckWifi()
        {
            address = "";
            adapterName = "";
            // get adapters
            NetworkInterface[] nis = NetworkInterface.GetAllNetworkInterfaces();
            foreach(var ni in nis)
            {
                // skip unenabled adapters
                if (ni.OperationalStatus != OperationalStatus.Up) continue;
                // skip adapters that are not ethernet or wifi
                if (ni.NetworkInterfaceType != NetworkInterfaceType.Ethernet &&
                    ni.NetworkInterfaceType != NetworkInterfaceType.Wireless80211) continue;
                // get IP properties
                var props = ni.GetIPProperties();
                // analyze addresses
                foreach(var addr in props.UnicastAddresses)
                {
                    // select IPv4 except loopback
                    if(addr.Address.AddressFamily == AddressFamily.InterNetwork &&
                        !IPAddress.IsLoopback(addr.Address) && addr.IsDnsEligible)
                    {
                        adapterName = ni.Name;
                        address = addr.Address.ToString();
                        DebugLog("CheckWifi: selected " + address);
                        break;
                    }
                }
                if (address.Length > 0) break;
            }
            // check if at least 1 address is found
            if (address.Length == 0)
                throw new ArgumentException("No valid IPv4 network (Wifi/Ethernet) found");
        }

        // debug log
        private void DebugLog(string message)
        {
            Debug.WriteLine(string.Format("[{0}] {1}", TAG, message));
        }
    }
}
