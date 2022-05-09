using System;
using System.Net;
using System.Net.Sockets;
using System.Net.NetworkInformation;

namespace Experimental
{
    partial class Program
    {
        static void listIPs()
        {
            var host = Dns.GetHostEntry(Dns.GetHostName());
            foreach(var ip in host.AddressList)
            {
                if(ip.AddressFamily == AddressFamily.InterNetwork)
                {
                    Console.WriteLine("IP: " + ip);
                }
            }
        }

        static void listIPs2()
        {
            NetworkInterface[] networkInterfaces = NetworkInterface.GetAllNetworkInterfaces();

            foreach (NetworkInterface network in networkInterfaces)
            {
                // Read the IP configuration for each network
                IPInterfaceProperties properties = network.GetIPProperties();

                // Each network interface may have multiple IP addresses
                foreach (var address in properties.UnicastAddresses)
                {
                    // We're only interested in IPv4 addresses for now
                    if (address.Address.AddressFamily != AddressFamily.InterNetwork)
                        continue;

                    // Ignore loopback addresses (e.g., 127.0.0.1)
                    if (IPAddress.IsLoopback(address.Address))
                        continue;

                    Console.WriteLine(address.Address.ToString() + " (" + network.Name + ") ");
                    Console.WriteLine("ID: " + network.Id);
                    Console.WriteLine("IsDnsEligible: " + address.IsDnsEligible);
                    Console.WriteLine("Is Up? " + (network.OperationalStatus == OperationalStatus.Up));
                    Console.WriteLine("Interface Type: " + network.NetworkInterfaceType.ToString());
                    Console.WriteLine();
                }
            }
        }
    }

}