// SPDX-License-Identifier: GPL-3.0

pragma solidity 0.8.24;

contract Agent {
    event Register (
        address indexed addr, 
        string url
    );

    event Unregister (
        address indexed addr
    );

    event AddCredit (
        address indexed addr, 
        uint256 inc_amount,
        uint256 result_amount
    );

    event Withdraw (
        address indexed addr, 
        uint256 amount
    );
    

    uint256 private balance;
    mapping(address => string) private register_agents;
    address[] agents_list;
    mapping(address => uint256) private  agent_credits;
    
    function register(string memory url) external {
        register_agents[msg.sender] = url;
        agents_list.push(msg.sender);
        emit Register(msg.sender, url);
    }

    function getUrl() external view returns (string memory) {
        require(agents_list.length > 0, "No agent available");

        bytes32 random_bytes = keccak256(abi.encodePacked(block.timestamp, blockhash(block.number-1)));
        uint256 index = uint256(uint8(random_bytes[0])) * agents_list.length / 256;
        return register_agents[agents_list[index]];
    }

    function unregister() external {
        delete register_agents[msg.sender];
        for (uint256 i = 0; i < agents_list.length; i++) {
            if (agents_list[i] == msg.sender) {
                delete agents_list[i];
            }
        }
        emit Unregister(msg.sender);
    }

    function increaseCredit(uint256 increment) external returns (uint256){
        require(bytes(register_agents[msg.sender]).length > 0, "You have not registered");

        uint256 credit = agent_credits[msg.sender];
        if (type(uint256).max - credit >= increment) {
            credit += increment;
        } else {
            credit = type(uint256).max;
        }
        agent_credits[msg.sender] = credit;

        emit AddCredit(msg.sender, increment, credit);
        return credit;
    }

    function getCredit() public view returns (uint256) {
        return agent_credits[msg.sender];
    }

    function deposit() external payable {
        balance += msg.value;
    }   

    function withdraw() external {
        require(agent_credits[msg.sender] > 0, "Nothing to withdraw");
        require(agent_credits[msg.sender] <= balance, "No enough funds");

        uint256 amount = agent_credits[msg.sender];
        agent_credits[msg.sender] = 0;
        balance -= amount;
        payable(msg.sender).transfer(amount);

        emit Withdraw(msg.sender, amount);
    }

}
