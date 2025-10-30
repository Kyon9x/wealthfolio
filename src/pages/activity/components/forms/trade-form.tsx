import {
  Card,
  CardContent,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
  MoneyInput,
  QuantityInput,
} from '@wealthfolio/ui';
import { useFormContext } from 'react-hook-form';
import { AccountSelectOption } from '../activity-form';
import {
  ActivityTypeSelector,
  type ActivityType as ActivityTypeUI,
} from '../activity-type-selector';
import { CashBalanceWarning } from '../cash-balance-warning';
import { AssetSymbolInput, CommonFields, ConfigurationCheckbox } from './common';

export const TradeForm = ({ accounts }: { accounts: AccountSelectOption[] }) => {
  const { control, watch, setValue } = useFormContext();
  const isManualAsset = watch('assetDataSource') === 'MANUAL';

  const tradeTypes: ActivityTypeUI[] = [
    {
      value: 'BUY',
      label: 'Buy',
      icon: 'ArrowDown',
      description:
        'Purchase an asset. This increases your holding quantity and decreases your cash balance.',
    },
    {
      value: 'SELL',
      label: 'Sell',
      icon: 'ArrowUp',
      description:
        'Sell an asset. This decreases your holding quantity and increases your cash balance.',
    },
  ];

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div className="flex-1">
          <ActivityTypeSelector control={control} types={tradeTypes} columns={2} />
        </div>
      </div>
      <CashBalanceWarning />
      <Card>
        <CardContent className="space-y-6 pt-2">
          <ConfigurationCheckbox showCurrencyOption={true} />
          <FormField
            control={control}
            name="assetId"
            render={({ field }) => (
              <AssetSymbolInput
                field={field}
                isManualAsset={isManualAsset}
                onDataSourceChange={(dataSource) => setValue('assetDataSource', dataSource)}
              />
            )}
          />
          <div className="flex space-x-4">
            <FormField
              control={control}
              name="quantity"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Shares</FormLabel>
                  <FormControl>
                    <QuantityInput {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <FormField
              control={control}
              name="unitPrice"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Price</FormLabel>
                  <FormControl>
                    <MoneyInput {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <FormField
              control={control}
              name="fee"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Fee</FormLabel>
                  <FormControl>
                    <MoneyInput {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
          </div>
          <CommonFields accounts={accounts} />
        </CardContent>
      </Card>
    </div>
  );
};
