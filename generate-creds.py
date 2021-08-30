#!/usr/bin/env python3

from ecdsa import SigningKey, NIST384p
import secrets
import shlex

for realm in ["DOORS", "MEMBER_PROJECTS", "DRINK"]:
  for key in ["READ", "AUTH", "UPDATE"]:
    print("export GK_REALM_" + realm + "_" + key + "_KEY=" + secrets.token_hex(16))

  # Generate ECDSA key pair
  sk = SigningKey.generate(curve=NIST384p)
  vk = sk.get_verifying_key()
  private_key = sk.to_pem().decode()
  public_key = vk.to_pem().decode()
  print("export GK_REALM_" + realm + "_PUBLIC_KEY=" + shlex.quote(public_key))
  print("export GK_REALM_" + realm + "_PRIVATE_KEY=" + shlex.quote(private_key))

print("export GK_SYSTEM_SECRET=" + secrets.token_hex(16))
