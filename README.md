# 🚫 Fully Fee-Less Substrate Node 🚫

This repository provides a Substrate-based blockchain node that implements a **fully feeless transaction model**. In this implementation, all transactions are free, but additional mechanisms are in place to ensure fair usage and prevent abuse. ✨

## Introduction

[Duniter Node](https://github.com/duniter/duniter-v2s) implements a **semi-feeless** transaction model that aims to reduce transaction costs while maintaining fairness and security. The core idea behind Duniter’s approach is to allow transactions to be free under normal usage conditions, with fees only being incurred when the system is overloaded. 🌍

This repository aims to provide a **fully feeless** Substrate-based blockchain node, where **all transactions are free** under all conditions. However, mechanisms are in place to ensure fairness and prevent misuse of the system. ⚖️

## Features

1. **Fully Fee-Less Transactions**  
   This implementation provides a **fully feeless transaction model**, where transactions are completely free for all accounts. No fees are charged for transactions, regardless of blockchain usage. 🚫💸

2. **Per-Account Rate Limiting**  
   While all transactions are free, each account is subject to a **rate limiter**. This limiter enforces a maximum number of free transactions per block, ensuring that no single account or a group of accounts can overload the network with excessive transaction requests. ⏳🔒

---

### TODO List
- [x] Custom `AccountData` and `AccountStore`
- [x] Rate limiter transaction extension
- [ ] Length limiter transaction extension
- [ ] Package in one standalone pallet
