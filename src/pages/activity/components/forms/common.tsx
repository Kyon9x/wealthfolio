import TickerSearchInput from '@/components/ticker-search';
import { Checkbox } from '@/components/ui/checkbox';
import { Input } from '@/components/ui/input';
import { Textarea } from '@/components/ui/textarea';
import { DataSource } from '@/lib/constants';
import {
  CurrencyInput,
  DatePickerInput,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@wealthfolio/ui';
import { useFormContext } from 'react-hook-form';
import { AccountSelectOption } from '../activity-form';

export interface ConfigurationCheckboxProps {
  showCurrencyOption?: boolean;
  shouldShowSymbolLookup?: boolean;
}

export const ConfigurationCheckbox = ({
  showCurrencyOption = true,
  shouldShowSymbolLookup = true,
}: ConfigurationCheckboxProps) => {
  const { control } = useFormContext();
  // Track the data source before switching to MANUAL
  const previousDataSourceRef = useRef<string>(DataSource.YAHOO);

  return (
    <div className="flex items-center justify-end space-x-6">
      {shouldShowSymbolLookup && (
        <FormField
          control={control}
          name="assetDataSource"
          render={({ field }) => {
            // Update the ref whenever the data source changes to a non-MANUAL value
            if (field.value !== DataSource.MANUAL) {
              previousDataSourceRef.current = field.value;
            }

            return (
              <FormItem className="mt-2 space-y-1">
                <div className="flex items-center justify-between">
                  <div className="flex items-center space-x-2">
                    <label
                      htmlFor="use-lookup-checkbox"
                      className="text-muted-foreground hover:text-foreground cursor-pointer text-sm"
                    >
                      Skip Symbol Lookup
                    </label>
                    <Checkbox
                      id="use-lookup-checkbox"
                      checked={field.value === DataSource.MANUAL}
                      onCheckedChange={(checked) => {
                        if (checked) {
                          // Save current data source before switching to MANUAL
                          if (field.value !== DataSource.MANUAL) {
                            previousDataSourceRef.current = field.value;
                          }
                          field.onChange(DataSource.MANUAL);
                        } else {
                          // Restore the previous data source
                          field.onChange(previousDataSourceRef.current);
                        }
                      }}
                      defaultChecked={field.value === DataSource.MANUAL}
                      className="h-4 w-4"
                    />
                  </div>
                </div>
              </FormItem>
            );
          }}
        />
      )}
      {showCurrencyOption && (
        <FormField
          control={control}
          name="showCurrencySelect"
          render={({ field }) => (
            <FormItem className="mt-2 space-y-1">
              <div className="flex items-center justify-between">
                <div className="flex items-center space-x-2">
                  <label
                    htmlFor="use-different-currency-checkbox"
                    className="text-muted-foreground hover:text-foreground cursor-pointer text-sm"
                  >
                    Use Different Currency
                  </label>
                  <Checkbox
                    id="use-different-currency-checkbox"
                    checked={field.value}
                    onCheckedChange={field.onChange}
                    className="h-4 w-4"
                  />
                </div>
              </div>
            </FormItem>
          )}
        />
      )}
    </div>
  );
};

export const CommonFields = ({ accounts }: { accounts: AccountSelectOption[] }) => {
  const { control, watch } = useFormContext();
  const showCurrency = watch('showCurrencySelect');

  return (
    <>
      <FormField
        control={control}
        name="accountId"
        render={({ field }) => (
          <FormItem>
            <FormLabel>Account</FormLabel>
            <FormControl>
              <Select onValueChange={field.onChange} defaultValue={field.value}>
                <SelectTrigger>
                  <SelectValue placeholder="Select an account" />
                </SelectTrigger>
                <SelectContent className="max-h-[500px] overflow-y-auto">
                  {accounts.map((account) => (
                    <SelectItem value={account.value} key={account.value}>
                      {account.label}
                      <span className="text-muted-foreground font-light">({account.currency})</span>
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </FormControl>
            <FormMessage />
          </FormItem>
        )}
      />
      <FormField
        control={control}
        name="activityDate"
        render={({ field }) => (
          <FormItem className="flex flex-col">
            <FormLabel>Date</FormLabel>
            <DatePickerInput
              onChange={(date: Date | undefined) => field.onChange(date)}
              value={field.value}
              disabled={field.disabled}
              enableTime={true}
              timeGranularity="minute"
            />
            <FormMessage />
          </FormItem>
        )}
      />
      {showCurrency && (
        <FormField
          control={control}
          name="currency"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Activity Currency</FormLabel>
              <FormControl>
                <CurrencyInput {...field} />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />
      )}
      <FormField
        control={control}
        name="comment"
        render={({ field }) => (
          <FormItem>
            <FormLabel>Description</FormLabel>
            <FormControl>
              <Textarea
                placeholder="Add an optional description or comment for this transaction..."
                className="resize-none"
                rows={3}
                {...field}
                value={field.value || ''}
              />
            </FormControl>
            <FormMessage />
          </FormItem>
        )}
      />
    </>
  );
};

export function AssetSymbolInput({
  field,
  isManualAsset,
  onDataSourceChange,
}: {
  field: any;
  isManualAsset: boolean;
  onDataSourceChange?: (dataSource: string) => void;
}) {
  return (
    <FormItem className="-mt-2">
      <FormLabel>Symbol</FormLabel>
      <FormControl>
        {isManualAsset ? (
          <Input
            placeholder="Enter symbol"
            className="h-10"
            {...field}
            onChange={(e) => field.onChange(e.target.value.toUpperCase())}
          />
        ) : (
          <TickerSearchInput
            onSelectResult={(symbol, dataSource) => {
              field.onChange(symbol);
              if (dataSource && onDataSourceChange) {
                onDataSourceChange(dataSource);
              }
            }}
            {...field}
          />
        )}
      </FormControl>
      <FormMessage className="text-xs" />
    </FormItem>
  );
}
