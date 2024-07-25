import { Disclosure } from "@headlessui/react";
import clsx from "clsx";
import { useAtomValue } from "jotai";
import { BellIcon } from "lucide-react";
import NavigationItem, { RouteItem } from "./NavigationItem";
import { userAtom } from "@/pages/atoms";

export interface NavProps {
  routes: RouteItem[];
}

export interface MobileNavProps extends NavProps {
  userMenu: RouteItem[];
}

export function MobileNav({ routes, userMenu }: MobileNavProps) {
  /** @TODO This shoud be deleted and place at top level when the back end supports users */
  const user = useAtomValue(userAtom);

  return (
    <Disclosure.Panel className="sm:hidden">
      <div className="space-y-1 pb-3 pt-2">
        {routes.map((item) => (
          <Disclosure.Button
            key={item.name}
            as="a"
            href={item.href}
            className={clsx(
              item.current
                ? "border-indigo-500 bg-indigo-50 text-indigo-700"
                : "border-transparent text-gray-600 hover:border-gray-300 hover:bg-gray-50 hover:text-gray-800",
              "block border-l-4 py-2 pl-3 pr-4 text-base font-medium",
            )}
            aria-current={item.current ? "page" : undefined}
          >
            {item.name}
          </Disclosure.Button>
        ))}
      </div>
      <div className="border-t border-gray-200 pb-3 pt-4">
        <div className="flex items-center px-4">
          <div className="shrink-0">
            <img
              className="size-10 rounded-full"
              src={user.imageUrl}
              alt=""
            />
          </div>
          <div className="ml-3">
            <div className="text-base font-medium text-gray-800">
              {user.name}
            </div>
            <div className="text-sm font-medium text-gray-500">
              {user.email}
            </div>
          </div>
          <button
            type="button"
            className="relative ml-auto shrink-0 rounded-full bg-white p-1 text-gray-400 hover:text-gray-500 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2"
          >
            <span className="absolute -inset-1.5" />
            <span className="sr-only">View notifications</span>
            <BellIcon className="size-6" aria-hidden="true" />
          </button>
        </div>
        <div className="mt-3 space-y-1">
          {userMenu.map((item) => (
            <Disclosure.Button
              key={item.name}
              as="a"
              href={item.href}
              className="block px-4 py-2 text-base font-medium text-gray-500 hover:bg-gray-100 hover:text-gray-800"
            >
              {item.name}
            </Disclosure.Button>
          ))}
        </div>
      </div>
    </Disclosure.Panel>
  );
}

export function Nav({ routes }: NavProps) {
  return routes.map((item) => <NavigationItem key={item.name} item={item} />);
}

export default Nav;
