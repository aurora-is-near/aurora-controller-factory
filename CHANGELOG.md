# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.3.3 2025-06-06

- Added a new role `Downgrader` for the `downgrade` method.

## 0.3.2 2025-03-11

- Added a possibility to provide arguments for the `delegate_pause` method.

## 0.3.1 2025-03-05

- Updated the `near-plugins` to 0.5.0.

## 0.3.0 2025-03-04

- Added the `Updater` role for the `up_stage_code` method.
- Removed not working view transactions: `get_release_blob` and `get_latest_release_blob`.
- Added the possibility to attach a deposit for staking storage in the `add_release_blob`.
- Fixed the issue with logging big size of arguments in the `delegate_execution`.

## 0.2.1 2024-12-02

- Reworked the `get_deployments` view method.

## 0.2.0 2024-12-02

- Update dependencies.
- Make transaction payable.
- Do not update deployment info after failed upgrade.
- Remove ownable functionality.
- Add a possibility to set amount of gas for migration.
- Use enum for collection prefixes.

## 0.1.0 2023-12-22

- Initial implementation. See the `README.md` for more details.  
