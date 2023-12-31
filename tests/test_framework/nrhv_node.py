import os
import shutil
import sys

from config.node_config import NRHV_CONFIG
from test_framework.blockchain_node import NodeType, TestNode
from config.node_config import MINER_ID
from utility.utils import (
    initialize_toml_config,
    p2p_port,
    rpc_port,
    blockchain_rpc_port,
)


class NrhvNode(TestNode):
    def __init__(
        self,
        index,
        root_dir,
        binary,
        updated_config,
        log_contract_address,
        mine_contract_address,
        log,
        rpc_timeout=10,
        libp2p_nodes=None,
    ):
        local_conf = NRHV_CONFIG.copy()
        if libp2p_nodes is None:
            if index == 0:
                libp2p_nodes = []
            else:
                libp2p_nodes = []
                for i in range(index):
                    libp2p_nodes.append(f"/ip4/127.0.0.1/tcp/{p2p_port(i)}")

        indexed_config = {
            "network_libp2p_port": p2p_port(index),
            "rpc_listen_address": f"127.0.0.1:{rpc_port(index)}",
            "network_libp2p_nodes": libp2p_nodes,
            "log_contract_address": log_contract_address,
            "mine_contract_address": mine_contract_address,
            "blockchain_rpc_endpoint": f"http://127.0.0.1:{blockchain_rpc_port(0)}",
        }
        # Set configs for this specific node.
        local_conf.update(indexed_config)
        # Overwrite with personalized configs.
        local_conf.update(updated_config)
        data_dir = os.path.join(root_dir, "nrhv_node" + str(index))
        rpc_url = "http://" + local_conf["rpc_listen_address"]
        super().__init__(
            NodeType.Nrhv,
            index,
            data_dir,
            rpc_url,
            binary,
            local_conf,
            log,
            rpc_timeout,
        )

    def setup_config(self):
        os.mkdir(self.data_dir)
        log_config_path = os.path.join(self.data_dir, self.config["log_config_file"])
        with open(log_config_path, "w") as f:
            f.write("debug")

        initialize_toml_config(self.config_file, self.config)

    def wait_for_rpc_connection(self):
        self._wait_for_rpc_connection(lambda rpc: rpc.nrhv_getStatus() is not None)

    def start(self):
        self.log.info("Start neurahive node %d", self.index)
        super().start()

    # rpc
    def nrhv_get_status(self):
        return self.rpc.nrhv_getStatus()["connectedPeers"]

    def nrhv_upload_segment(self, segment):
        return self.rpc.nrhv_uploadSegment([segment])

    def nrhv_download_segment(self, data_root, start_index, end_index):
        return self.rpc.nrhv_downloadSegment([data_root, start_index, end_index])

    def nrhv_get_file_info(self, data_root):
        return self.rpc.nrhv_getFileInfo([data_root])

    def nrhv_get_file_info_by_tx_seq(self, tx_seq):
        return self.rpc.nrhv_getFileInfoByTxSeq([tx_seq])

    def shutdown(self):
        self.rpc.admin_shutdown()
        self.wait_until_stopped()

    def admin_start_sync_file(self, tx_seq):
        return self.rpc.admin_startSyncFile([tx_seq])

    def admin_get_sync_status(self, tx_seq):
        return self.rpc.admin_getSyncStatus([tx_seq])

    def sycn_status_is_completed_or_unknown(self, tx_seq):
        status = self.rpc.admin_getSyncStatus([tx_seq])
        return status == "Completed" or status == "unknown"

    def clean_data(self):
        shutil.rmtree(os.path.join(self.data_dir, "db"))
