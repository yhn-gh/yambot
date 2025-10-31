# Yambot

Yambot is a chatbot for Twitch IRC written in Rust.

## Features

todo

## Configuration

Update `\src\backend\twitch\auth.rs` and set client_id and client_secret in order to make bot connect to twitch chat.

## Usage

todo

## Required scopes
In order for bot to handle everything you need scopes:
- user:read:chat
- channel:bot
- user:bot
- channel:moderate
- user:write:chat

You can use to https://yamii.bieda.it/ to generate access token.

*Building app yourself requires you to generate access token with client_id set in auth.rs*

## Contributing

If you have any ideas, suggestions, or bug reports, please open an issue or submit a pull request on the [GitHub repository](https://github.com/xyamii/yambot).


