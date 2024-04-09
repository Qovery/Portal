import { dialogOpenedAtomFamily } from "@/atoms/dialog.atoms";
import { DialogIds } from "@/enums/dialog-ids.enum";
import { ThemeColors } from "@/enums/theme-colors.enum";
import { Dialog as HeadlessDialog, Transition } from "@headlessui/react";
import { XMarkIcon } from "@heroicons/react/24/outline";
import { useAtom } from "jotai";
import { Fragment, ReactNode } from "react";
import { Button } from "./Button";

export interface DialogProps {
  id: DialogIds;
  title: string;
  children: ReactNode;
}

export default function Dialog({ id, title, children }: DialogProps) {
  const [isOpen, setIsOpen] = useAtom(dialogOpenedAtomFamily(id));

  return (
    <Transition.Root show={isOpen} as={Fragment}>
      <HeadlessDialog as="div" className="relative z-10" onClose={setIsOpen}>
        <Transition.Child
          as={Fragment}
          enter="ease-out duration-300"
          enterFrom="opacity-0"
          enterTo="opacity-100"
          leave="ease-in duration-200"
          leaveFrom="opacity-100"
          leaveTo="opacity-0"
        >
          <div className="fixed inset-0 bg-gray-500 bg-opacity-75 transition-opacity" />
        </Transition.Child>

        <div className="fixed inset-0 z-10 w-screen overflow-y-auto">
          <div className="flex min-h-full items-end justify-center p-4 text-center sm:items-center sm:p-0">
            <Transition.Child
              as={Fragment}
              enter="ease-out duration-300"
              enterFrom="opacity-0 translate-y-4 sm:translate-y-0 sm:scale-95"
              enterTo="opacity-100 translate-y-0 sm:scale-100"
              leave="ease-in duration-200"
              leaveFrom="opacity-100 translate-y-0 sm:scale-100"
              leaveTo="opacity-0 translate-y-4 sm:translate-y-0 sm:scale-95"
            >
              <HeadlessDialog.Panel className="relative transform overflow-hidden rounded-lg bg-white px-4 pb-4 pt-5 text-left shadow-xl transition-all sm:my-8 sm:w-full sm:max-w-lg sm:p-6">
                <div className="absolute right-0 top-0 hidden pr-4 pt-4 sm:block">
                  <button
                    type="button"
                    className="rounded-md bg-white text-gray-400 hover:text-gray-500 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2"
                    onClick={() => setIsOpen(false)}
                  >
                    <span className="sr-only">Close</span>
                    <XMarkIcon className="h-6 w-6" aria-hidden="true" />
                  </button>
                </div>
                <div className="sm:flex sm:items-start">
                  <div className="mt-3 text-center sm:ml-4 sm:mt-0 sm:text-left">
                    <HeadlessDialog.Title
                      as="h3"
                      className="text-base font-semibold leading-6 text-gray-900"
                    >
                      {title}
                    </HeadlessDialog.Title>
                    <div className="mt-2">{children}</div>
                  </div>
                </div>
                <div className="mt-5 sm:mt-4 sm:flex sm:flex-row-reverse">
                  <Button
                    type="button"
                    color={ThemeColors.PRIMARY}
                    onClick={() => setIsOpen(false)}
                  >
                    Validate
                  </Button>
                  <Button
                    type="button"
                    flat
                    color={ThemeColors.PRIMARY}
                    onClick={() => setIsOpen(false)}
                  >
                    Close
                  </Button>
                </div>
              </HeadlessDialog.Panel>
            </Transition.Child>
          </div>
        </div>
      </HeadlessDialog>
    </Transition.Root>
  );
}
