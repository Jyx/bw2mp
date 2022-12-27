#!/usr/bin/env python
import argparse
import pprint
import sys
import json


def load_json(file):
    js = None
    with open(file, 'r') as j:
        js = json.load(j)
    return js


def print_yml(file):
    cfg = pprint.pformat(json)
    logging.debug(cfg)


def get_args():
    parser = argparse.ArgumentParser(description='Bitwarden to Moolitpass\n')

    parser.add_argument('-f', '--file', action='store', required=False,
                        default=None,
                        help='Bitwarden exported json file')

    parser.add_argument('--filter', action='store', required=False,
                        default=None,
                        help='Filter out a single folder')

    parser.add_argument('-e', '--exclude', action='store', required=False,
                        default=None,
                        help='Exclude a folder')

    if len(sys.argv) < 2:
        parser.print_help()
        sys.exit(1)

    return parser.parse_args()


def folder_name_to_id(js, name):
    for f in js['folders']:
        if f['name'] == name:
            return f['id']


def main():
    print("Bitwarden to Mooltipass")
    args = get_args()

    if not args.file:
        print("Error: Need a json file from Bitwarden")
        exit()

    js = load_json(args.file)

    folder_id = None
    if args.filter:
        folder_id = folder_name_to_id(js, args.filter)

    exclude = None
    if args.exclude:
        exclude = folder_name_to_id(js, args.exclude)

    with open(f"{args.file}.csv", 'w') as csv:
        for i in js['items']:
            folderid = i.get('folderId', None)
            if folderid and exclude:
                if folderid in exclude:
                    # print(f"Excluding {args.exclude}/{exclude}")
                    continue

            login = i.get('login', None)
            if login is None:
                # print(f"Couldn't find login for: {i}")
                continue

            username = login['username']
            password = login['password']

            for uri in login['uris']:
                item = f"{uri['uri']},{username},{password}"
                if folder_id:
                    if folder_id in i['folderId']:
                        print(item)
                        csv.write(f"{item}\n")
                else:
                    print(item)
                    csv.write(f"{item}\n")


if __name__ == "__main__":
    main()
