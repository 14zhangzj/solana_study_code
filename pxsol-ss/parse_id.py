#!/usr/bin/env python3
"""
快速解析 id.json 私钥
用法: python parse_id.py [文件路径]
"""

import json
import base58
import sys
import os

# 默认路径
default_path = os.path.expanduser("~/.config/solana/id.json")

# 获取文件路径
if len(sys.argv) > 1:
    file_path = sys.argv[1]
else:
    file_path = default_path

try:
    # 读取文件
    with open(file_path, 'r') as f:
        data = json.load(f)

    print(f"📁 解析文件: {file_path}")

    # 提取私钥（前32字节）
    if isinstance(data, list) and len(data) >= 32:
        private_key_bytes = bytes(data[:32])
        private_key = base58.b58encode(private_key_bytes).decode()

        # 提取公钥（如果有）
        if len(data) >= 64:
            public_key_bytes = bytes(data[32:64])
            public_key = base58.b58encode(public_key_bytes).decode()
            print(f"📍 公钥: {public_key}")

        print(f"🔑 私钥: {private_key}")
        print(f"📏 长度: {len(private_key_bytes)} 字节")

        # 验证长度
        if len(private_key_bytes) == 32:
            print("✅ 私钥长度正确 (32 字节)")
        else:
            print(f"⚠️  警告: 期望32字节，实际{len(private_key_bytes)}字节")

        # 生成使用命令
        print(f"\n💡 使用以下命令部署:")
        print(f'python make.py --prikey "{private_key}" deploy')

    else:
        print(f"❌ 错误: 文件格式不正确或长度不足")

except FileNotFoundError:
    print(f"❌ 错误: 文件不存在: {file_path}")
except json.JSONDecodeError:
    print(f"❌ 错误: 无法解析 JSON 文件")
except Exception as e:
    print(f"❌ 错误: {e}")