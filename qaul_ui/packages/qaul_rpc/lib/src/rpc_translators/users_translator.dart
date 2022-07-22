part of 'abstract_rpc_module_translator.dart';


class UsersTranslator extends RpcModuleTranslator {
  @override
  Modules get type => Modules.USERS;

  @override
  Future<RpcTranslatorResponse?> decodeMessageBytes(List<int> data) async {
    final message = Users.fromBuffer(data);
    switch (message.whichMessage()) {
      case Users_Message.userList:
        final users = message
            .ensureUserList()
            .user
            .map((u) => User(
          name: u.name,
          idBase58: u.idBase58,
          id: Uint8List.fromList(u.id),
          key: Uint8List.fromList(u.key),
          keyType: u.keyType,
          keyBase58: u.keyBase58,
          isBlocked: u.blocked,
          isVerified: u.verified,
        ))
            .toList();

        return RpcTranslatorResponse(type, users);
      case Users_Message.userUpdate:
        final userEntry = message.ensureUserUpdate();
        final user = User(
          name: userEntry.name,
          idBase58: userEntry.idBase58,
          id: Uint8List.fromList(userEntry.id),
          key: Uint8List.fromList(userEntry.key),
          keyType: userEntry.keyType,
          keyBase58: userEntry.keyBase58,
          isBlocked: userEntry.blocked,
          isVerified: userEntry.verified,
          status: _mapFrom(userEntry.connectivity),
        );
        return RpcTranslatorResponse(type, user);
      default:
        return super.decodeMessageBytes(data);
    }
  }

  ConnectionStatus _mapFrom(Connectivity c) {
    if (c == Connectivity.Online) return ConnectionStatus.online;
    if (c == Connectivity.Reachable) return ConnectionStatus.reachable;
    return ConnectionStatus.offline;
  }
}
