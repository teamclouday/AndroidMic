using AndroidMic.Audio;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Net.Sockets;
using System.Text;
using System.Threading.Tasks;

namespace AndroidMic.Streaming
{
    public class StreamerAdb : Streamer
    {
        private readonly string TAG = "StreamerAdb";

        private readonly AdbServer mServer = new AdbServer();
        private readonly AdbClient mAdbClient = new AdbClient();
        private DeviceData mDevice = null;
        private TcpClient mTcpClient = null;

        private string adapterName;
        private readonly Socket listener;
        private Socket client;
        private string address;
        private int port = 6000;

        private bool isConnectionAllowed = false;

        public StreamerAdb()
        {
            CheckAdb();
            listener = new Socket(AddressFamily.InterNetwork, SocketType.Stream, ProtocolType.Tcp);
            BindPort();
            listener.Listen(5);
            Status = ServerStatus.LISTENING;
            DebugLog("Server Started");
            isConnectionAllowed = true;
            listener.BeginAccept(new AsyncCallback(AcceptCallback), listener);
        }

        public override string GetClientInfo()
        {
            throw new NotImplementedException();
        }

        public override string GetServerInfo()
        {
            throw new NotImplementedException();
        }

        public override bool IsAlive()
        {
            throw new NotImplementedException();
        }

        public override Task Process(AudioBuffer sharedBuffer)
        {
            throw new NotImplementedException();
        }

        public override void Shutdown()
        {
            throw new NotImplementedException();
        }
    }
}
